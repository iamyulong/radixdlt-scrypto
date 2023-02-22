use crate::errors::{ModuleError, RuntimeError};
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelModuleApi, LockFlags};
use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::api::types::{ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NodeMoveError {
    CantMoveDownstream(RENodeId),
    CantMoveUpstream(RENodeId),
}

#[derive(Debug, Clone)]
pub struct NodeMoveModule {}

impl NodeMoveModule {
    fn prepare_move_downstream<Y: KernelModuleApi<RuntimeError>>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Proof(..) => {
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Proof(ProofOffset::Info),
                    LockFlags::MUTABLE,
                )?;
                let mut substate_ref_mut = api.kernel_get_substate_ref_mut(handle)?;
                let proof = substate_ref_mut.proof_info();

                let rtn = if proof.restricted {
                    Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                        NodeMoveError::CantMoveDownstream(node_id),
                    )))
                } else {
                    proof.change_to_restricted();
                    Ok(())
                };

                api.kernel_drop_lock(handle)?;

                rtn
            }
            RENodeId::Bucket(..) | RENodeId::Component(..) => Ok(()),

            RENodeId::TransactionRuntime
            | RENodeId::AuthZoneStack
            | RENodeId::Logger
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Vault(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Identity(..)
            | RENodeId::Validator(..)
            | RENodeId::Clock(..)
            | RENodeId::Global(..)
            | RENodeId::Account(..)
            | RENodeId::AccessController(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(node_id)),
            )),
        }
    }

    fn prepare_move_upstream<Y: KernelModuleApi<RuntimeError>>(
        node_id: RENodeId,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Bucket(..)
            | RENodeId::Proof(..)
            | RENodeId::Component(..)
            | RENodeId::Vault(..)
            | RENodeId::Account(..) => Ok(()),

            RENodeId::TransactionRuntime
            | RENodeId::AuthZoneStack
            | RENodeId::Logger
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Identity(..)
            | RENodeId::Validator(..)
            | RENodeId::Clock(..)
            | RENodeId::Global(..)
            | RENodeId::AccessController(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveUpstream(node_id)),
            )),
        }
    }
}

impl KernelModule for NodeMoveModule {
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _actor: &Option<ResolvedActor>,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_downstream(*node_id, api)?;
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<ResolvedActor>,
        call_frame_update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_upstream(*node_id, api)?;
        }

        Ok(())
    }
}
