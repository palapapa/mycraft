use assets_manager::source::*;
use assets_manager::*;
use assets_manager::Error as AssetCacheError;
use bitflags::*;
use log::*;
use wesl::*;
use wesl::Error as WeslError;
use wesl::syntax::*;
use std::path::*;
use std::fmt::Display;
use std::fmt;
use std::string::*;
use thiserror::*;
use crate::constants::*;

/// A convenience function to load a [`WeslShader`] from an [`AssetCache`]. See
/// [`WeslShader`] for more detail. This function simply merges each feature
/// flag in `features` with hyphens, and append that to `id` after appending a
/// dot. The `id` parameter should not contain the feature flags to use.
pub fn load_wesl_shader<'cache>(asset_cache: &'cache AssetCache, id: &str, features: ShaderFeatures) -> Result<&'cache Handle<WeslShader>, AssetCacheError> {
    let id_with_features = format!("{id}.{}", features.hyphen_separated_string());
    asset_cache.load::<WeslShader>(&id_with_features)
}

/// An abstraction of WESL shader source code strings. Since a WESL shader needs
/// to be compiled using [`Wesl`] to be converted into WGSL and be used by
/// [`wgpu`], [`Asset`] is implemented for this struct so that a WESL shader is
/// automatically compiled when loaded from the disk.
/// 
/// When you load this type with [`AssetCache::load`], the `id` parameter you
/// pass need to be formatted in a special way. The `id` string doubles as a
/// specification for what WESL feature flags (that you use inside `@if`s) to
/// enable when compiling it to WGSL. If the shader is located at
/// `<ASSET_PATH>/path/to/shader.wesl`, then the `id` should be
/// `path.to.shader.<feature1>-<feature2>-...-<featureN>`. Each <feature> is the
/// name of the feature flag to enable. Any other feature flags that are used in
/// the shader will be disabled. If you want to disable all feature flags, you
/// still must have a trailing dot in the ID.
/// 
/// You can save yourself from constructing the special ID string by using the
/// [`load_wesl_shader`] convenience function.
pub struct WeslShader {
    wesl_compile_result: CompileResult
}

impl Asset for WeslShader {
    fn load(cache: &AssetCache, id: &SharedString) -> Result<Self, BoxedError> {
        info!("Loading WESL shader: {id}");
        // Uses the assets base path as the base path for shader module
        // resolution. This means that if you put your shaders in the
        // assets/shaders/ directory, absolute imports should start with
        // package::shader::...
        let Some(shader_base_path) = get_assets_base_path(cache) else {
            error!("Error while loading shader {id}: {:#?}", WeslShaderLoadingError::CannotLocateShaderBasePath);
            return Err(WeslShaderLoadingError::CannotLocateShaderBasePath.into())
        };
        let mut compiler = Wesl::new(shader_base_path);
        let Some((_, features_string)) = id.rsplit_once('.') else {
            error!("Error while loading shader {id}: {:#?}", WeslShaderLoadingError::CannotGetFeatureList);
            return Err(WeslShaderLoadingError::CannotGetFeatureList.into());
        };
        // If features_string is empty, features will contain a single empty string.
        let features = features_string.split('-');
        compiler.set_features(features.map(|feature| (feature, Feature::Enable)));
        compiler.add_constants(SHADER_CONSTANTS.iter().copied());
        let compile_result = match compiler.compile(&ModulePath::new(PathOrigin::Absolute, id.split('.').map(ToString::to_string).collect())) {
            Ok(compile_result) => compile_result,
            Err(err) => {
                error!("Error while loading shader {id}: {err:#?}");
                return Err(WeslShaderLoadingError::WeslCompilation(err).into());
            }
        };
        // Although WESL doesn't need them, we still need to load the shader
        // module and its dependencies from `cache` so that assets_manager knows
        // about them and can do hot-reloading.
        for module_path in &compile_result.modules {
            // ModulePath::origin can only be Absolute or Package, and if its
            // the latter, we won't bother enabling hot-reloading for it since
            // the module is in another package.
            if module_path.origin != PathOrigin::Absolute {
                continue;
            }
            #[expect(clippy::unwrap_used, reason = "Since module_path.origin is PathOrigin::Absolute, its string representation must start with \"package::\".")]
            let asset_id = module_path.to_string().strip_prefix("package::").unwrap().replace("::", ".");
            info!("Registering the WESL module {asset_id} as a dependency for hot-reloading.");
            // Reading the file just to register it as a dependency.
            drop(cache.source().read(&asset_id, "wesl"));
        }
        Ok(Self { wesl_compile_result: compile_result })
    }
}

impl Display for WeslShader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.wesl_compile_result.fmt(f)
    }
}

/// Gets the absolute path to the assets directory. The [`Source`] that `cache`
/// uses must be [`FileSystem`]; otherwise, returns [`None`].
fn get_assets_base_path(cache: &AssetCache) -> Option<&Path> {
    cache.downcast_raw_source::<FileSystem>().map(FileSystem::root)
}

#[derive(Debug, Error)]
pub enum WeslShaderLoadingError {
    #[error("The directory to use as the base for shader module resolution cannot be located because the AssetCache used doesn't use a FileSystem Source.")]
    CannotLocateShaderBasePath,
    #[error(transparent)]
    WeslCompilation(#[from] WeslError),
    #[error("The id passed to Asset::load for WeslShader always need to contain at least one dot to denote the shader feature flag list. If you want to disable all feature flags, you still must have a trailing dot in the ID.")]
    CannotGetFeatureList
}

bitflags! {
    /// A bit flags type that represents a set of shader feature flags, i.e.
    /// `@if()` attributes. To use a feature flag in a shader, put the name of
    /// the feature flag inside the parentheses of the `@if()` attribute. The
    /// names of the feature flags can be obtained by calling
    /// [`Self::single_feature_name`].
    #[derive(Clone, Copy, Default)]
    pub struct ShaderFeatures: u32 {
        const HAS_POSITION_ATTRIBUTES = 1 << 0;
        const HAS_UV_ATTRIBUTES = 1 << 1;
        const HAS_COLOR_ATTRIBUTES = 1 << 2;
    }
}

impl ShaderFeatures {
    /// Returns the name of the one feature flag that `self` contains. This is
    /// the thing that you should put inside `@if` in the shader. `self` must
    /// contain exactly one feature; otherwise, [`None`] is returned. Unknown
    /// bits are ignored.
    pub fn single_feature_name(self) -> Option<&'static str> {
        let mut it = self.iter_names();
        let (name, _) = it.next()?;
        if it.next().is_some() {
            return None;
        }
        Some(name)
    }

    /// Returns a string that contains the name of each enabled shader feature,
    /// separated by hyphens. The order of the shader features in the returned
    /// string is consistent. An empty string is returned if there are no
    /// features enabled. Unknown bits are ignored.
    /// 
    /// This is a helper mainly used for the [`Asset::load`] implementation of
    /// [`WeslShader`].
    pub fn hyphen_separated_string(self) -> String {
        let names: Vec<&'static str> = self.iter_names().map(|(name, _)| name).collect();
        names.join("-")
    }
}

/// A type used to identify a shader.
pub struct ShaderId {
    /// The [`AssetCache`] ID of the shader. Note that this is different from
    /// the ID needed by the [`Asset`] implementation for [`WeslShader`]. This
    /// should not contain the shader features to use. You should specify those in [`Self::features`].
    pub asset_id: String,
    
    /// The shader features to use.
    pub features: ShaderFeatures
}
