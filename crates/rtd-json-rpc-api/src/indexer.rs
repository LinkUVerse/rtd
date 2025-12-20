// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::{RpcResult, SubscriptionResult};
use jsonrpsee::proc_macros::rpc;

use rtd_json_rpc_types::RtdTransactionBlockEffects;
use rtd_json_rpc_types::{
    DynamicFieldPage, EventFilter, EventPage, ObjectsPage, Page, RtdEvent, RtdObjectResponse,
    RtdObjectResponseQuery, RtdTransactionBlockResponseQuery, TransactionBlocksPage,
    TransactionFilter,
};
use rtd_open_rpc_macros::open_rpc;
use rtd_types::base_types::{ObjectID, RtdAddress};
use rtd_types::digests::TransactionDigest;
use rtd_types::dynamic_field::DynamicFieldName;
use rtd_types::event::EventID;

#[open_rpc(namespace = "rtdx", tag = "Extended API")]
#[rpc(server, client, namespace = "rtdx")]
pub trait IndexerApi {
    /// Return the list of objects owned by an address.
    /// Note that if the address owns more than `QUERY_MAX_RESULT_LIMIT` objects,
    /// the pagination is not accurate, because previous page may have been updated when
    /// the next page is fetched.
    /// Please use rtdx_queryObjects if this is a concern.
    #[method(name = "getOwnedObjects")]
    async fn get_owned_objects(
        &self,
        /// the owner's Rtd address
        address: RtdAddress,
        /// the objects query criteria.
        query: Option<RtdObjectResponseQuery>,
        /// An optional paging cursor. If provided, the query will start from the next item after the specified cursor. Default to start from the first item if not specified.
        cursor: Option<ObjectID>,
        /// Max number of items returned per page, default to [QUERY_MAX_RESULT_LIMIT] if not specified.
        limit: Option<usize>,
    ) -> RpcResult<ObjectsPage>;

    /// Return list of transactions for a specified query criteria.
    #[method(name = "queryTransactionBlocks")]
    async fn query_transaction_blocks(
        &self,
        /// the transaction query criteria.
        query: RtdTransactionBlockResponseQuery,
        /// An optional paging cursor. If provided, the query will start from the next item after the specified cursor. Default to start from the first item if not specified.
        cursor: Option<TransactionDigest>,
        /// Maximum item returned per page, default to QUERY_MAX_RESULT_LIMIT if not specified.
        limit: Option<usize>,
        /// query result ordering, default to false (ascending order), oldest record first.
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionBlocksPage>;

    /// Return list of events for a specified query criteria.
    #[method(name = "queryEvents")]
    async fn query_events(
        &self,
        /// The event query criteria. See [Event filter](https://docs.rtd.io/build/event_api#event-filters) documentation for examples.
        query: EventFilter,
        /// optional paging cursor
        cursor: Option<EventID>,
        /// maximum number of items per page, default to [QUERY_MAX_RESULT_LIMIT] if not specified.
        limit: Option<usize>,
        /// query result ordering, default to false (ascending order), oldest record first.
        descending_order: Option<bool>,
    ) -> RpcResult<EventPage>;

    /// Subscribe to a stream of Rtd event
    #[subscription(name = "subscribeEvent", item = RtdEvent)]
    fn subscribe_event(
        &self,
        /// The filter criteria of the event stream. See [Event filter](https://docs.rtd.io/build/event_api#event-filters) documentation for examples.
        filter: EventFilter,
    ) -> SubscriptionResult;

    /// Subscribe to a stream of Rtd transaction effects
    #[subscription(name = "subscribeTransaction", item = RtdTransactionBlockEffects)]
    fn subscribe_transaction(&self, filter: TransactionFilter) -> SubscriptionResult;

    /// Return the list of dynamic field objects owned by an object.
    #[method(name = "getDynamicFields")]
    async fn get_dynamic_fields(
        &self,
        /// The ID of the parent object
        parent_object_id: ObjectID,
        /// An optional paging cursor. If provided, the query will start from the next item after the specified cursor. Default to start from the first item if not specified.
        cursor: Option<ObjectID>,
        /// Maximum item returned per page, default to [QUERY_MAX_RESULT_LIMIT] if not specified.
        limit: Option<usize>,
    ) -> RpcResult<DynamicFieldPage>;

    /// Return the dynamic field object information for a specified object
    #[method(name = "getDynamicFieldObject")]
    async fn get_dynamic_field_object(
        &self,
        /// The ID of the queried parent object
        parent_object_id: ObjectID,
        /// The Name of the dynamic field
        name: DynamicFieldName,
    ) -> RpcResult<RtdObjectResponse>;

    /// Return the resolved address given resolver and name
    #[method(name = "resolveNameServiceAddress")]
    async fn resolve_name_service_address(
        &self,
        /// The name to resolve
        name: String,
    ) -> RpcResult<Option<RtdAddress>>;

    /// Return the resolved names given address,
    /// if multiple names are resolved, the first one is the primary name.
    #[method(name = "resolveNameServiceNames")]
    async fn resolve_name_service_names(
        &self,
        /// The address to resolve
        address: RtdAddress,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
    ) -> RpcResult<Page<String, ObjectID>>;
}
