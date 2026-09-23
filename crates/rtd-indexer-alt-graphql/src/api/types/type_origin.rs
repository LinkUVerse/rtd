// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::api::scalars::rtd_address::RtdAddress;
use async_graphql::Object;
use rtd_types::move_package::TypeOrigin as NativeTypeOrigin;

pub struct TypeOrigin {
    native: NativeTypeOrigin,
}

/// Information about which previous versions of a package introduced its types.
#[Object]
impl TypeOrigin {
    /// Module defining the type.
    pub(crate) async fn module(&self) -> Option<String> {
        Some(self.native.module_name.clone())
    }

    /// Name of the struct.
    #[graphql(name = "struct")]
    pub(crate) async fn struct_(&self) -> Option<String> {
        Some(self.native.datatype_name.clone())
    }

    /// The storage ID of the package that first defined this type.
    pub(crate) async fn defining_id(&self) -> Option<RtdAddress> {
        Some(self.native.package.into())
    }
}

impl From<NativeTypeOrigin> for TypeOrigin {
    fn from(native: NativeTypeOrigin) -> Self {
        Self { native }
    }
}
