use crate::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    global::dxb_block::{DXBBlock, HeaderParsingError},
    stdlib::vec::Vec,
    task::spawn_with_panic_notify,
};
use core::prelude::rust_2024::*;
use std::async_iter::AsyncIterator;
use log::{error, info};

#[derive(Debug)]
pub struct BlockCollector {
    // The current block being received.
    current_block: Vec<u8>,

    // The specified length of the current block being received, if known.
    current_block_specified_length: Option<u16>,
}

/// Implements the logic to collect DXB blocks from incoming byte slices.
impl BlockCollector {
    async fn receive_slice(&mut self, slice: &[u8]) -> Option<DXBBlock> {
        // Add the received data to the current block.
        self.current_block.extend_from_slice(slice);

        while !self.current_block.is_empty() {
            // Extract the block length from the header if it is not already known.
            if self.current_block_specified_length.is_none() {
                let length_result =
                    DXBBlock::extract_dxb_block_length(&self.current_block);

                match length_result {
                    Ok(length) => {
                        self.current_block_specified_length = Some(length);
                    }
                    Err(HeaderParsingError::InsufficientLength) => {
                        break;
                    }
                    Err(err) => {
                        error!("Received invalid block header: {err:?}");
                        self.current_block.clear();
                        self.current_block_specified_length = None;
                    }
                }
            }

            // If the block length is specified and the current block is long enough, extract the block.
            if let Some(specified_length) = self.current_block_specified_length
            {
                if self.current_block.len() >= specified_length as usize {
                    let block_slice = self
                        .current_block
                        .drain(0..specified_length as usize)
                        .collect::<Vec<u8>>();

                    let block_result = DXBBlock::from_bytes(&block_slice).await;

                    match block_result {
                        Ok(block) => {
                            self.current_block_specified_length = None;
                            return Some(block);
                        }
                        Err(err) => {
                            error!("Received invalid block header: {err:?}");
                            self.current_block.clear();
                            self.current_block_specified_length = None;
                        }
                    }
                } else {
                    break;
                }
            }
            // otherwise, wait for more data
            else {
                break;
            }
        }
        None
    }

    /// Returns a sender that accepts incoming byte slices and
    /// an async iterator that yields DXB blocks collected from incoming byte slices.
    pub fn create() -> (UnboundedSender<Vec<u8>>, impl AsyncIterator<Item = DXBBlock>) {
        let (bytes_in_sender, bytes_in_receiver) = create_unbounded_channel();
        let block_collector = BlockCollector {
            current_block: Vec::new(),
            current_block_specified_length: None,
        };

        (
            bytes_in_sender,
            run_block_collector_task(block_collector, bytes_in_receiver),
        )
    }
}

pub fn run_block_collector_task(mut block_collector: BlockCollector, mut bytes_in_receiver: UnboundedReceiver<Vec<u8>>) -> impl AsyncIterator<Item = DXBBlock> {
    async gen move {
        info!("BlockCollector task started");
        while let Some(slice) = bytes_in_receiver.next().await {
            if let Some(block) = block_collector.receive_slice(&slice).await {
                yield block;
            };
        }
    }
}
