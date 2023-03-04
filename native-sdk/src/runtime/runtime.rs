use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ClientApi, ClientObjectApi};
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::transaction_runtime::*;
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::time::*;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(EPOCH_MANAGER.into()),
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(CLOCK.into()),
            CLOCK_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockGetCurrentTimeInput { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_compare_against_current_time<Y, E>(
        api: &mut Y,
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> Result<bool, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(CLOCK.into()),
            CLOCK_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockCompareCurrentTimeInput {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::TransactionRuntime,
            TRANSACTION_RUNTIME_GENERATE_UUID_IDENT,
            scrypto_encode(&TransactionRuntimeGenerateUuid {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
