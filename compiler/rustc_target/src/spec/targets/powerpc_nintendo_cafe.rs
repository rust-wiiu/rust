use rustc_abi::Endian;

use crate::spec::{
    Arch, Cc, CodeModel, LinkerFlavor, Lld, Os, RelocModel, Target, TargetMetadata,
    TargetOptions,
};

// RPX files require code, data and loader sections at fixed addresses.
const LINKER_SCRIPT: &str = include_str!("./powerpc_nintendo_cafe_linker_script.ld");

pub(crate) fn target() -> Target {
    // Relocations are kept in the ELF because the ELF -> RPX conversion needs them.
    let pre_link_args = TargetOptions::link_args(
        LinkerFlavor::Gnu(Cc::No, Lld::No),
        &["--emit-relocs", "-znocopyreloc"],
    );

    Target {
        llvm_target: "powerpc-none-eabi".into(),
        metadata: TargetMetadata {
            description: Some("PowerPC Nintendo Wii U, Cafe OS".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true),
        },
        pointer_width: 32,
        data_layout: "E-m:e-p:32:32-Fn32-i64:64-n32".into(),
        arch: Arch::PowerPC,
        options: TargetOptions {
            endian: Endian::Big,
            os: Os::Cafe,
            vendor: "nintendo".into(),
            cpu: "750".into(),
            features: "+hard-float,+fpu,+longcall".into(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),
            link_script: Some(LINKER_SCRIPT.into()),
            pre_link_args,
            relocation_model: RelocModel::Static,
            code_model: Some(CodeModel::Large),
            max_atomic_width: Some(32),
            supports_stack_protector: false,
            exe_suffix: ".elf".into(),
            emit_debug_gdb_scripts: false,
            ..Default::default()
        },
    }
}
