#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

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
        .with_async_function("request", proxy_request)?
        .build_readonly()
}

async fn proxy_socket(_: Lua, (proxy, url): (String, String)) -> LuaResult<Websocket<WsStream>> {
    let proxy = proxy.parse().into_lua_err()?;
    let url = url.parse().into_lua_err()?;
    let stream = WsStream::connect_url_via_socks5(&proxy, url).await?;
    Ok(Websocket::from(stream))
}

async fn proxy_request(lua: Lua, (proxy, config): (String, LuaValue)) -> LuaResult<LuaValue> {
    let config = match config {
        LuaValue::Table(table) => {
            let opts: LuaTable = table
                .get::<Option<LuaTable>>("options")?
                .unwrap_or_else(|| lua.create_table().unwrap());
            opts.set("proxy", proxy)?;
            table.set("options", opts)?;
            LuaValue::Table(table)
        }
        LuaValue::String(url) => {
            let tbl = lua.create_table()?;
            tbl.set("url", url)?;
            let opts = lua.create_table()?;
            opts.set("proxy", proxy)?;
            tbl.set("options", opts)?;
            LuaValue::Table(tbl)
        }
        other => other,
    };

    let net: LuaTable = lua.load("return require('@lune/net')").eval()?;
    let req_func: LuaFunction = net.get("request")?;
    req_func.call_async(config).await
}
