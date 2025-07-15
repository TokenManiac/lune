#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;
use url::Url;

use lune_utils::TableBuilder;
use lune_std_net::{WsStream, Websocket};

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `proxy` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `proxy` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_async_function("socket", proxy_socket)?
        .build_readonly()
}

async fn proxy_socket(_: Lua, (proxy, url): (String, String)) -> LuaResult<Websocket<WsStream>> {
    let proxy = proxy.parse().into_lua_err()?;
    let url = url.parse().into_lua_err()?;
    let stream = WsStream::connect_url_via_socks5(&proxy, url).await?;
    Ok(Websocket::from(stream))
}
