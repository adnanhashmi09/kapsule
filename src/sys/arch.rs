use anyhow::{Error, Ok, Result};
use serde_json::Value;

use std::{ffi::CStr, mem};

#[derive(Debug)]
pub enum Arch {
    ARM64(Variant),
    ARM32(Variant),
    AMD32(Variant),
    AMD64,
    UNKNOWN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    V5,
    V6,
    V7,
    V8,
    I386,
    I586,
    I686,
}

impl Variant {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "v5" => Some(Self::V5),
            "v6" => Some(Self::V6),
            "v7" => Some(Self::V7),
            "v8" => Some(Self::V8),
            "386" => Some(Self::I386),
            "586" => Some(Self::I586),
            "686" => Some(Self::I686),
            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Variant::V5 => "v5",
            Variant::V6 => "v6",
            Variant::V7 => "v7",
            Variant::V8 => "v8",
            Variant::I386 => "386",
            Variant::I586 => "586",
            Variant::I686 => "686",
        }
    }
}

impl Arch {
    pub fn from_str(arch: &str) -> Self {
        match arch {
            "arm64" | "aarch64" => Self::ARM64(Variant::V8),
            "armv5l" => Self::ARM32(Variant::V5),
            "armv6l" => Self::ARM32(Variant::V6),
            "armv7l" => Self::ARM32(Variant::V7),
            "i386" => Self::AMD32(Variant::I386),
            "i586" => Self::AMD32(Variant::I586),
            "i686" => Self::AMD32(Variant::I686),
            "x86_64" | "amd64" => Self::AMD64,
            _ => Self::UNKNOWN,
        }
    }

    pub fn to_string(&self) -> Option<&'static str> {
        match self {
            Arch::ARM64(_) => Some("arm64"),
            Arch::ARM32(_) => Some("arm"),
            Arch::AMD64 => Some("amd64"),
            Arch::AMD32(variant) => match variant {
                Variant::I386 => Some("386"),
                Variant::I586 => Some("586"),
                Variant::I686 => Some("686"),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn get_variant(&self) -> Option<Variant> {
        match self {
            Arch::ARM64(v) => Some(*v),
            Arch::ARM32(v) => Some(*v),
            Arch::AMD32(v) => Some(*v),
            _ => None,
        }
    }
}
