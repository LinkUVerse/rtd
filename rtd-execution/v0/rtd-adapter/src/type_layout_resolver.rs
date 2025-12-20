// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::programmable_transactions::context::new_session_for_linkage;
use crate::programmable_transactions::{
    context::load_type,
    linkage_view::{LinkageInfo, LinkageView},
};
use move_core_types::annotated_value as A;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_vm_runtime::{move_vm::MoveVM, session::Session};
use rtd_types::base_types::ObjectID;
use rtd_types::error::{RtdErrorKind, RtdResult};
use rtd_types::execution::TypeLayoutStore;
use rtd_types::storage::{BackingPackageStore, PackageObject};
use rtd_types::{error::RtdError, layout_resolver::LayoutResolver};

/// Retrieve a `MoveStructLayout` from a `Type`.
/// Invocation into the `Session` to leverage the `LinkageView` implementation
/// common to the runtime.
pub struct TypeLayoutResolver<'state, 'vm> {
    session: Session<'state, 'vm, LinkageView<'state>>,
}

/// Implements RtdResolver traits by providing null implementations for module and resource
/// resolution and delegating backing package resolution to the trait object.
struct NullRtdResolver<'state>(Box<dyn TypeLayoutStore + 'state>);

impl<'state, 'vm> TypeLayoutResolver<'state, 'vm> {
    pub fn new(vm: &'vm MoveVM, state_view: Box<dyn TypeLayoutStore + 'state>) -> Self {
        let session = new_session_for_linkage(
            vm,
            LinkageView::new(Box::new(NullRtdResolver(state_view)), LinkageInfo::Unset),
        );
        Self { session }
    }
}

impl LayoutResolver for TypeLayoutResolver<'_, '_> {
    fn get_annotated_layout(
        &mut self,
        struct_tag: &StructTag,
    ) -> Result<A::MoveDatatypeLayout, RtdError> {
        let type_tag: TypeTag = TypeTag::from(struct_tag.clone());
        let Ok(ty) = load_type(&mut self.session, &type_tag) else {
            return Err(RtdErrorKind::FailObjectLayout {
                st: format!("{}", struct_tag),
            }
            .into());
        };
        let layout = self.session.type_to_fully_annotated_layout(&ty);
        let Ok(A::MoveTypeLayout::Struct(layout)) = layout else {
            return Err(RtdErrorKind::FailObjectLayout {
                st: format!("{}", struct_tag),
            }
            .into());
        };
        Ok(A::MoveDatatypeLayout::Struct(layout))
    }
}

impl BackingPackageStore for NullRtdResolver<'_> {
    fn get_package_object(&self, package_id: &ObjectID) -> RtdResult<Option<PackageObject>> {
        self.0.get_package_object(package_id)
    }
}
