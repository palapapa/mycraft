use assets_manager::source::*;
use assets_manager::*;
use log::*;
use wesl::*;
use wesl::Error as WeslError;
use wesl::syntax::*;
use std::path::*;
use std::fmt::Display;
use std::fmt;
use std::string::*;
use thiserror::*;

/// An abstraction of WESL shader source code strings. This is return type used
/// by [`crate::material::Material`] when they return shader code. Since a WESL
/// shader needs to be compiled using [`wesl::Wesl`] to be converted into WGSL
/// and be used by [`wgpu`], [`Asset`] is implemented for this struct so that a
/// WESL shader is automatically compiled when loaded from the disk.
pub struct WeslShader {
    pub wesl_compile_result: CompileResult
}

impl Asset for WeslShader {
    fn load(cache: &AssetCache, id: &SharedString) -> Result<Self, BoxedError> {
        info!("Loading WESL shader: {id}");
        // Uses the assets base path as the base path for shader module
        // resolution. This means that if you put your shaders in the
        // assets/shaders/ directory, absolute imports should start with
        // package::shader::...
        let Some(shader_base_path) = get_assets_base_path(cache) else {
            return Err(WeslShaderLoadError::CannotLocateShaderBasePath.into())
        };
        let compiler = Wesl::new(shader_base_path);
        let compile_result = match compiler.compile(&ModulePath::new(PathOrigin::Absolute, id.split('.').map(ToString::to_string).collect())) {
            Ok(compile_result) => compile_result,
            Err(err) => return Err(WeslShaderLoadError::WeslCompilation(err).into())
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

/// Uses the ID of an asset and returns an absolute path to the asset. If the
/// [`Source`] used by `cache` is not [`FileSystem`], returns [`None`].
fn asset_cache_id_to_path(cache: &AssetCache, id: &SharedString) -> Option<PathBuf> {
    let assets_base_path = get_assets_base_path(cache)?;
    let result = assets_base_path.join(id.split('.').collect::<PathBuf>());
    Some(result)
}

#[derive(Debug, Error)]
pub enum WeslShaderLoadError {
    #[error("The directory to use as the base for shader module resolution cannot be located because the AssetCache used doesn't use a FileSystem Source.")]
    CannotLocateShaderBasePath,
    #[error(transparent)]
    WeslCompilation(#[from] WeslError)
}
