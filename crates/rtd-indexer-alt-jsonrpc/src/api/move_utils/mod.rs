// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use rtd_json_rpc_types::RtdMoveNormalizedFunction;
use rtd_open_rpc::Module;
use rtd_open_rpc_macros::open_rpc;
use rtd_types::base_types::ObjectID;

use crate::context::Context;

use super::rpc_module::RpcModule;

mod error;
mod response;

#[open_rpc(namespace = "rtd", tag = "Move APIs")]
#[rpc(server, namespace = "rtd")]
trait MoveApi {
    #[method(name = "getNormalizedMoveFunction")]
    async fn get_normalized_move_function(
        &self,
        package: ObjectID,
        module_name: String,
        function_name: String,
    ) -> RpcResult<RtdMoveNormalizedFunction>;
}

pub(crate) struct MoveUtils(pub Context);

#[async_trait::async_trait]
impl MoveApiServer for MoveUtils {
    async fn get_normalized_move_function(
        &self,
        package: ObjectID,
        module_name: String,
        function_name: String,
    ) -> RpcResult<RtdMoveNormalizedFunction> {
        let Self(ctx) = self;
        Ok(response::function(ctx, package, &module_name, &function_name).await?)
    }
}

impl RpcModule for MoveUtils {
    fn schema(&self) -> Module {
        MoveApiOpenRpc::module_doc()
    }

    fn into_impl(self) -> jsonrpsee::RpcModule<Self> {
        self.into_rpc()
    }
}
