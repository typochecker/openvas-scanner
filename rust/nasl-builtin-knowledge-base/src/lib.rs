// SPDX-FileCopyrightText: 2023 Greenbone AG
//
// SPDX-License-Identifier: GPL-2.0-or-later WITH x11vnc-openssl-exception

use std::time::{SystemTime, UNIX_EPOCH};

use nasl_builtin_utils::{error::FunctionErrorKind, NaslFunction};
use nasl_function_proc_macro::nasl_function;
use storage::{Field, Kb, Retrieve};

use nasl_builtin_utils::{Context, Register};
use nasl_syntax::NaslValue;

/// NASL function to set a knowledge base
#[nasl_function(named(name, value, expires))]
fn set_kb_item(
    c: &Context,
    name: &str,
    value: NaslValue,
    expires: Option<u64>,
) -> Result<NaslValue, FunctionErrorKind> {
    let expires = expires.map(|seconds| {
        let start = SystemTime::now();
        match start.duration_since(UNIX_EPOCH) {
            Ok(x) => x.as_secs() + seconds as u64,
            Err(_) => 0,
        }
    });
    c.dispatcher()
        .dispatch(
            c.key(),
            Field::KB(Kb {
                key: name.to_string(),
                value: value.clone().as_primitive(),
                expire: expires,
            }),
        )
        .map(|_| NaslValue::Null)
        .map_err(|e| e.into())
}

/// NASL function to get a knowledge base
#[nasl_function]
fn get_kb_item(arg: &NaslValue, c: &Context) -> Result<NaslValue, FunctionErrorKind> {
    c.retriever()
        .retrieve(c.key(), Retrieve::KB(arg.to_string()))
        .map(|r| {
            r.into_iter()
                .filter_map(|x| match x {
                    Field::NVT(_) | Field::NotusAdvisory(_) | Field::Result(_) => None,
                    Field::KB(kb) => Some(kb.value.into()),
                })
                .collect::<Vec<_>>()
        })
        .map(NaslValue::Fork)
        .map_err(|e| e.into())
}

/// Returns found function for key or None when not found
pub fn lookup(key: &str) -> Option<NaslFunction> {
    match key {
        "set_kb_item" => Some(set_kb_item),
        "get_kb_item" => Some(get_kb_item),
        _ => None,
    }
}

pub struct KnowledgeBase;

impl nasl_builtin_utils::SyncNaslFunctionExecuter for KnowledgeBase {
    fn nasl_fn_execute(
        &self,
        name: &str,
        register: &Register,
        context: &Context,
    ) -> Option<nasl_builtin_utils::NaslResult> {
        lookup(name).map(|x| x(register, context))
    }

    fn nasl_fn_defined(&self, name: &str) -> bool {
        lookup(name).is_some()
    }
}
