use std::{env, path::PathBuf, sync::LazyLock};

use anyhow::{Result, bail};
use async_fs as fs;
use mlua::Compiler as LuaCompiler;

pub static CURRENT_EXE: LazyLock<PathBuf> =
    LazyLock::new(|| env::current_exe().expect("failed to get current exe"));
const MAGIC: &[u8; 8] = b"cr3sc3nt";

/*
    TODO: Right now all we do is append the bytecode to the end
    of the binary, but we will need a more flexible solution in
    the future to store many files as well as their metadata.

    The best solution here is most likely to use a well-supported
    and rust-native binary serialization format with a stable
    specification, one that also supports byte arrays well without
    overhead, so the best solution seems to currently be Postcard:

    https://github.com/jamesmunns/postcard
    https://crates.io/crates/postcard
*/

/**
    Metadata for a standalone Lune executable. Can be used to
    discover and load the bytecode contained in a standalone binary.
*/
#[derive(Debug, Clone)]
pub struct Metadata {
    pub bytecode: Vec<u8>,
    pub chunk_name: String,
}

impl Metadata {
    /**
        Returns whether or not the currently executing Lune binary
        is a standalone binary, and if so, the bytes of the binary.
    */
    pub async fn check_env() -> (bool, Vec<u8>) {
        let contents = fs::read(CURRENT_EXE.to_path_buf())
            .await
            .unwrap_or_default();
        let is_standalone = contents.ends_with(MAGIC);
        (is_standalone, contents)
    }

    /**
        Creates a patched standalone binary from the given script contents.
    */
    pub async fn create_env_patched_bin(
        base_exe_path: PathBuf,
        script_contents: impl Into<Vec<u8>>,
        chunk_name: impl AsRef<str>,
    ) -> Result<Vec<u8>> {
        let compiler = LuaCompiler::new()
            .set_optimization_level(2)
            .set_coverage_level(0)
            .set_debug_level(1);

        let mut patched_bin = fs::read(base_exe_path).await?;

        // Compile luau input into bytecode
        let bytecode = compiler.compile(script_contents.into())?;

        // Append the bytecode / metadata to the end
        let meta = Self { bytecode, chunk_name: chunk_name.as_ref().to_string() };
        patched_bin.extend_from_slice(&meta.to_bytes());

        Ok(patched_bin)
    }

    /**
        Tries to read a standalone binary from the given bytes.
    */
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self> {
        let bytes = bytes.as_ref();
        if bytes.len() < 24 || !bytes.ends_with(MAGIC) {
            bail!("not a standalone binary")
        }

        // Extract sizes
        let bytecode_size_bytes = &bytes[bytes.len() - 16..bytes.len() - 8];
        let chunk_name_size_bytes = &bytes[bytes.len() - 24..bytes.len() - 16];

        let bytecode_size =
            usize::try_from(u64::from_be_bytes(bytecode_size_bytes.try_into().unwrap()))?;
        let chunk_name_size =
            usize::try_from(u64::from_be_bytes(chunk_name_size_bytes.try_into().unwrap()))?;

        // Extract contents
        let chunk_end = bytes.len() - 24;
        let chunk_start = chunk_end - chunk_name_size;
        let bytecode_start = chunk_start - bytecode_size;

        let chunk_name = String::from_utf8(bytes[chunk_start..chunk_end].to_vec())?;
        let bytecode = bytes[bytecode_start..chunk_start].to_vec();

        Ok(Self { bytecode, chunk_name })
    }

    /**
        Writes the metadata chunk to a byte vector, to later bet read using `from_bytes`.
    */
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.bytecode);
        bytes.extend_from_slice(self.chunk_name.as_bytes());
        bytes.extend_from_slice(&(self.chunk_name.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&(self.bytecode.len() as u64).to_be_bytes());
        bytes.extend_from_slice(MAGIC);
        bytes
    }
}
