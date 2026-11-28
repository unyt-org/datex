use super::{
    DXB, dxb_parsed::ParsedDXBBlock, dxb_raw::RawDXBBlock, dxb_trait::DxB,
};
use crate::compiler::compile_script;

use crate::global::protocol_structures::{
    block_header::BlockHeader,
    encrypted_header::EncryptedHeader,
    routing_header::{EncryptionType, RoutingHeader, SignatureType},
};

use crate::prelude::*;
use core::pin::Pin;

use crate::{
    compiler::{CompileOptions, error::SimpleOrDetailedCompilerError},
    runtime::Runtime,
};

#[derive(Debug)]
pub enum BlockBuilderError {
    Placeholder,
}

pub struct DXBBuilder {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,
    pub encrypted_header: Option<EncryptedHeader>,
    pub signature: Option<Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl DXBBuilder {
    pub fn new() -> DXBBuilder {
        let routing_header = RoutingHeader::default();
        let block_header = BlockHeader::default();
        DXBBuilder {
            routing_header,
            block_header,
            encrypted_header: None,
            signature: None,
            body: None,
        }
    }

    pub fn build(&self) -> DXB {
        unimplemented!();
    }

    pub fn build_raw(&self, body: &[u8]) -> RawDXBBlock {
        RawDXBBlock {
            routing_header: self.routing_header.clone(),
            block_header: self.block_header.clone(),
            body: body.to_vec(),
        }
    }

    pub fn build_parsed(
        self,
        body: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = ParsedDXBBlock>>> {
        Box::pin(async move {
            ParsedDXBBlock {
                routing_header: self.routing_header.clone(),
                block_header: self.block_header.clone(),
                encrypted_header: self
                    .encrypted_header
                    .unwrap_or(EncryptedHeader::default())
                    .clone(),
                signature: self.signature.clone(),
                body: body.clone(),
            }
        })
    }

    // usage:
    // let body = builder::compile_body_from_script(dxscript)
    // builder.set_compiled_body(body)
    pub fn compile_body_from_script(
        datex_script: &str,
        runtime: Runtime,
    ) -> Result<Vec<u8>, SimpleOrDetailedCompilerError> {
        let (body, _) =
            compile_script(datex_script, CompileOptions::default(), runtime)?;

        Ok(body)
    }

    pub fn set_compiled_body(mut self, body: &[u8]) -> DXBBuilder {
        self.body = Some(body.to_vec());
        self
    }

    // usage:
    // builder.set_and_compile_body_from_script(dxscript)
    pub fn set_and_compile_body_from_script(
        mut self,
        dxscript: &str,
        runtime: Runtime,
    ) -> Result<DXBBuilder, BlockBuilderError> {
        let compiled_body = Self::compile_body_from_script(dxscript, runtime)
            .map_err(|_| BlockBuilderError::Placeholder)?;
        self.body = Some(compiled_body);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        compiler::compile_block,
        runtime::{CustomRuntimeInfo, RuntimeConfig, RuntimeRunner},
    };

    async fn get_runner() -> RuntimeRunner {
        RuntimeRunner::new(
            RuntimeConfig::default(),
            CustomRuntimeInfo::default(),
        )
        .await
    }

    #[tokio::test]
    pub async fn builder_debug() {
        let dxscript = r"1 + 2";
        let runner = get_runner().await;

        // the old way
        let old_dxb_block =
            compile_block(dxscript, runner.runtime.clone()).unwrap();

        // new ways
        let body = DXBBuilder::compile_body_from_script(
            dxscript,
            runner.runtime.clone(),
        )
        .unwrap();
        let raw_dxb_block = DXBBuilder::new().build_raw(&body).to_bytes();
        let parsed_dxb_block =
            DXBBuilder::new().build_parsed(body).await.to_bytes();

        let x = old_dxb_block.len() % 2;
        let y = raw_dxb_block.len() % 2;

        assert_eq!(old_dxb_block, parsed_dxb_block);
        assert_eq!(x, y + 1);
    }
}
