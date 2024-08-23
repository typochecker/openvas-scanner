// SPDX-FileCopyrightText: 2024 Greenbone AG
//
// SPDX-License-Identifier: GPL-2.0-or-later WITH x11vnc-openssl-exception

mod frame_forgery;
mod packet_forgery;
mod raw_ip_utils;
use frame_forgery::FrameForgery;
use nasl_builtin_utils::{combine_function_sets, NaslVars};

pub struct RawIp;

impl nasl_builtin_utils::NaslVarDefiner for RawIp {
    fn nasl_var_define(&self) -> NaslVars {
        let mut raw_ip_vars = packet_forgery::expose_vars();
        raw_ip_vars.extend(frame_forgery::expose_vars());
        raw_ip_vars
    }
}

combine_function_sets! {
    RawIp,
    (
        PacketForgery,
        FrameForgery,
    )
}
