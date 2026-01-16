use crate::components::core::*;
use bevy_ecs::change_detection::*;
use bevy_ecs::entity::*;
use bevy_ecs::hierarchy::*;
use bevy_ecs::query::*;
use bevy_ecs::removal_detection::*;
use bevy_ecs::system::*;
use log::*;

/// For [`Entity`]s whose [`GlobalTransformComponent`] or that of any of its
/// descendants have potentially changed, uses change detection to mark the
/// [`TransformTreeChangedComponent`] of it as changed so that
/// [`propagate_parent_transforms_system`] can skip traversing the [`Entity`]s
/// whose [`GlobalTransformComponent`] definitely doesn't need to be
/// recalculated.
/// 
/// Preciesly, for [`Entity`]s who satisfy any of the following:
/// 
/// - Its [`TransformComponent`] have changed this frame.
/// - It has been reparented.
/// - [`GlobalTransformComponent`] has been added to it. This criterion is
///   required even though [`TransformComponent`] requires
///   [`GlobalTransformComponent`], because that only guarantees that
///   [`GlobalTransformComponent`] is added when [`TransformComponent`] is
///   added, but not that [`GlobalTransformComponent`] is always present when
///   [`TransformComponent`] is. For example, [`GlobalTransformComponent`] can
///   be removed from an [`Entity`] and then added back.
/// - It has been orphaned.
/// 
/// This system sets the [`TransformTreeChangedComponent`]s of the [`Entity`]
/// and all its parents all the way up to the root to changed. This allows
/// [`propagate_parent_transforms_system`] to skip visiting a subtree when it is
/// certain that the subtree couldn't have any of its
/// [`GlobalTransformComponent`] changed.
pub fn mark_dirty_trees_system(
    global_transform_potentially_changed_entities: Query<'_,'_,
        Entity,
        Or<(Changed<TransformComponent>, Changed<ChildOf>, Added<GlobalTransformComponent>)>
    >,
    mut orphaned_entities: RemovedComponents<'_, '_, ChildOf>,
    mut child_of_and_ttcc: Query<'_, '_,
        (Option<&ChildOf>, &mut TransformTreeChangedComponent)
    >
) {
    for global_transform_potentially_changed_entity in global_transform_potentially_changed_entities.iter().chain(orphaned_entities.read()) {
        let mut next_entity_to_mark_as_dirty = global_transform_potentially_changed_entity;
        loop {
            let (child_of, mut ttcc) = match child_of_and_ttcc.get_mut(next_entity_to_mark_as_dirty) {
                Ok(value) => value,
                Err(err) => {
                    warn_malformed_hierarchy(&err);
                    break;
                }
            };
            if ttcc.is_changed() && !ttcc.is_added() {
                // Every eneity in global_transform_potentially_changed_entities
                // travels towards the root, marking every entity in between
                // (including both ends) as dirty. This means if this condition
                // is true, some entity has marked the rest of the path for us
                // and we can stop now. The !ttcc.is_added() condition is needed
                // because is_changed includes is_added, and we don't want to
                // skip entities who are just added this frame and haven't been
                // marked as dirty yet.
                break;
            }
            ttcc.set_changed();
            if let Some(parent) = child_of.map(ChildOf::parent) {
                next_entity_to_mark_as_dirty = parent;
            }
            else {
                break;
            }
        }
    }
}

/// This system updates those [`GlobalTransformComponent`]s that have changed,
/// either because their own [`TransformComponent`]s changed or that of some of
/// their parents changed.
/// 
/// This system uses the information provided by [`mark_dirty_trees_system`] to
/// skip traversing subtrees that definitely don't have any
/// [`GlobalTransformComponent`] that needs to be updated. Specifically:
/// 
/// 1. All trees whose [`TransformTreeChangedComponent`] on root [`Entity`]
///    isn't `is_changed` are skipped because none of the [`Entity`]s in it have
///    changed.
/// 2. For the other trees, run a depth first search on each tree. For each
///    [`Entity`] visited:
///     1. Recalculate its [`GlobalTransformComponent`] from the parent
///        [`GlobalTransformComponent`] and its [`TransformComponent`]. If this
///        is the root, [`GlobalTransformComponent`] is calculated only from the
///        root's [`TransformComponent`].
///     2. Compare the recalculated one with the current one.
///     3. If they are the same, traverse into only those children that have
///        their [`TransformTreeChangedComponent`] set to `is_changed`.
///     4. Otherwise, update the current [`GlobalTransformComponent`] and
///        traverse into all children.
/// 
/// To prevent step 2.2 from repeatedly comparing [`GlobalTransformComponent`]s
/// that are certain to be different when an [`Entity`] close to the root have
/// changed, a `bool` `any_parent_changed` value is propogated along the stack.
/// If `any_parent_changed` is `true`, we can skip directly to step 2.4 without
/// comparing.
pub fn propagate_parent_transforms_system(
    mut potentially_changed_roots: Query<'_, '_,
        Entity,
        (Without<ChildOf>, Changed<TransformTreeChangedComponent>)
    >,
    node_query: Query<'_, '_,
        (&TransformComponent, &mut GlobalTransformComponent, Option<&Children>)
    >,
    ttcc_query: Query<'_, '_,
        Ref<'_, TransformTreeChangedComponent>
    >
) {
    struct StackFrame<'a> {
        current_entity: Entity,
        /// If any of [`Self::current_entity`]'s parents all the way to the root
        /// have changed its [`GlobalTransformComponent`]. If this is `true`, we
        /// can directly recalculate [`Self::current_entity`]'s
        /// [`GlobalTransformComponent`] and visit all children without checking
        /// if we should skip any subtree.
        /// 
        /// This optimization means that if a parent's [`TransformComponent`]
        /// changed, but a child's [`TransformComponent`] also changed in a way
        /// that exactly cancels out the parent's change, it's
        /// [`GlobalTransformComponent`] will still be marked as `is_changed`
        /// even though it is the same.
        any_parent_changed: bool,
        /// [`None`] of [`Self::current_entity`] is the root.
        parent_global_transform: Option<&'a GlobalTransformComponent>
    }
    potentially_changed_roots.par_iter_mut().for_each(|root_entity| {
        let mut stack: Vec<StackFrame<'_>> = Vec::new();
        stack.push(StackFrame { current_entity: root_entity, any_parent_changed: false, parent_global_transform: None });
        while let Some(current_frame) = stack.pop() {
            // SAFETY: The closure passed to for_each must be Clone, which means
            // that all captured variables must also be Clone, but &mut
            // node_query is not Clone, so we need get_unchecked to avoid
            // mutably borrowing node_query here. This is safe because although
            // technically two threads can potentially mutate the same
            // `GlobalTransformComponent` on the same entity, since we are
            // parallelizing across roots, and all entities in a World should
            // form a forest instead of a graph, as long as the entity hierarchy
            // isn't inconsistent, no entity can have more than one parent, so
            // no two threads should be able to access the same entity.
            //
            // Bevy's implementation of transform propagation
            // https://github.com/bevyengine/bevy/blob/706fdd9800c4bf335d67f127ff6e5a93deadabb9/crates/bevy_transform/src/systems.rs#L183
            // actually explicitly guards against an inconsistent hierarchy,
            // where the entity D has a ChildOf that points to B, but C has a
            // Children that contains D. AFAIK this situation shouldn't be
            // possible to create with safe code, so it is not checked here.
            let (transform, mut global_transform, children) = match unsafe { node_query.get_unchecked(current_frame.current_entity) } {
                Ok((transform, global_transform, children)) => (transform, global_transform, children),
                Err(err) => {
                    warn_malformed_hierarchy(&err);
                    return;
                }
            };
            if current_frame.any_parent_changed {
                let parent_global_transform = current_frame.parent_global_transform.expect("any_parent_changed is true but current_entity is the root. This is impossible.");
                *global_transform = parent_global_transform.mul_transform(transform);
                let Some(children_unwrapped) = children else { continue };
                // We can't write `parent_global_transform:
                // Some(&global_transform)` because that way the reference can't
                // outlive global_transform, which is of type Mut. We need to
                // convert the Mut into Ref first and then call into_inner to
                // convert it into a normal reference with 'world lifetime.
                let global_transform_ref = Ref::from(global_transform).into_inner();
                for child in children_unwrapped {
                    stack.push(StackFrame { any_parent_changed: true, current_entity: *child, parent_global_transform: Some(global_transform_ref) });
                }
                continue;
            }
            let new_global_transform = current_frame.parent_global_transform.map_or_else(|| transform.into(), |parent_global_transform| parent_global_transform.mul_transform(transform));
            if new_global_transform == *global_transform {
                let Some(children_unwrapped) = children else { continue };
                let global_transform_ref = Ref::from(global_transform).into_inner();
                for child in children_unwrapped {
                    let ttcc = match ttcc_query.get(*child) {
                        Ok(ttcc) => ttcc,
                        Err(err) => {
                            warn_malformed_hierarchy(&err);
                            return;
                        }
                    };
                    if !ttcc.is_changed() {
                        continue;
                    }
                    stack.push(StackFrame { any_parent_changed: false, current_entity: *child, parent_global_transform: Some(global_transform_ref) });
                }
                continue;
            }
            *global_transform = new_global_transform;
            let Some(children_unwrapped) = children else { continue };
            let global_transform_ref = Ref::from(global_transform).into_inner();
            for child in children_unwrapped {
                stack.push(StackFrame { any_parent_changed: true, current_entity: *child, parent_global_transform: Some(global_transform_ref) });
            }
        }
    });
}

fn warn_malformed_hierarchy(err: &QueryEntityError) {
    warn!("Malformed transform hierarchy. TransformComponent, GlobalTransformComponent, and TransformTreeChangedComponent must always come in groups, \
        and all descendents of any Entity that have these three components should also have these three components. Error: {err:#?}");
}
