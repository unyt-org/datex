use crate::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    global::dxb_block::{DXBBlock, HeaderParsingError},
};

use crate::prelude::*;
use core::async_iter::AsyncIterator;
use log::{error, info};

#[derive(Debug)]
pub struct BlockCollector {
    // The current block being received.
    current_partial_block: Vec<u8>,

    // The specified length of the current block being received, if known.
    current_block_specified_length: Option<u16>,
}

/// Implements the logic to collect DXB blocks from incoming byte slices.
impl BlockCollector {
    async fn receive_slice(&mut self, slice: &[u8]) -> Option<DXBBlock> {
        info!("Receive slice: {:?}", slice);
        // Add the received data to the current block.
        self.current_partial_block.extend_from_slice(slice);

        while !self.current_partial_block.is_empty() {
            // Extract the block length from the header if it is not already known.
            if self.current_block_specified_length.is_none() {
                let length_result = DXBBlock::extract_dxb_block_length(
                    &self.current_partial_block,
                );

                match length_result {
                    Ok(length) => {
                        self.current_block_specified_length = Some(length);
                    }
                    Err(HeaderParsingError::InsufficientLength) => {
                        break;
                    }
                    Err(HeaderParsingError::InvalidMagicNumber) => {
                        error!(
                            "Received invalid block header: Invalid Magic Number"
                        );
                        self.current_partial_block.clear();
                        self.current_block_specified_length = None;
                    }
                }
            }

            // If the block length is specified and the current block is long enough, extract the block.
            if let Some(specified_length) = self.current_block_specified_length
            {
                if self.current_partial_block.len() >= specified_length as usize
                {
                    let block_slice = self
                        .current_partial_block
                        .drain(0..specified_length as usize)
                        .collect::<Vec<u8>>();

                    let block_result = DXBBlock::from_bytes(&block_slice);

                    match block_result {
                        Ok(block) => {
                            self.current_block_specified_length = None;
                            return Some(block);
                        }
                        Err(err) => {
                            error!("Received invalid block header: {err:?}");
                            self.current_partial_block.clear();
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
    pub fn create() -> (
        UnboundedSender<Vec<u8>>,
        impl AsyncIterator<Item = DXBBlock>,
    ) {
        let (bytes_in_sender, bytes_in_receiver) = create_unbounded_channel();
        let block_collector = BlockCollector {
            current_partial_block: Vec::new(),
            current_block_specified_length: None,
        };

        (
            bytes_in_sender,
            run_block_collector_task(block_collector, bytes_in_receiver),
        )
    }
}

pub fn run_block_collector_task(
    mut block_collector: BlockCollector,
    mut bytes_in_receiver: UnboundedReceiver<Vec<u8>>,
) -> impl AsyncIterator<Item = DXBBlock> {
    async gen move {
        info!("BlockCollector task started");
        while let Some(slice) = bytes_in_receiver.next().await {
            if let Some(block) = block_collector.receive_slice(&slice).await {
                yield block;
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::global::protocol_structures::routing_header::SignatureType;

    use super::*;

    #[tokio::test]
    async fn test_receive_complete_block() {
        let mut block_collector = BlockCollector {
            current_partial_block: Vec::new(),
            current_block_specified_length: None,
        };

        let mut block = DXBBlock::new_with_body(b"TestBody");
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::None);
        let block_bytes = block.to_bytes();

        let received_block = block_collector.receive_slice(&block_bytes).await;
        assert!(received_block.is_some());
        assert_eq!(received_block.unwrap().body, b"TestBody");
    }

    #[tokio::test]
    async fn test_receive_block_in_slices() {
        let mut block_collector = BlockCollector {
            current_partial_block: Vec::new(),
            current_block_specified_length: None,
        };

        let mut block = DXBBlock::new_with_body(b"TestBody");
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::None);
        let block_bytes = block.to_bytes();
        let part1 = &block_bytes[0..5]; // contains full magic number and block length
        let part2 = &block_bytes[5..];

        assert!(block_collector.receive_slice(part1).await.is_none());

        assert_eq!(block_collector.current_partial_block.len(), part1.len());
        assert_eq!(
            block_collector.current_block_specified_length,
            Some(block_bytes.len() as u16)
        );

        let received_block = block_collector.receive_slice(part2).await;
        assert!(received_block.is_some());
        assert_eq!(received_block.unwrap().body, b"TestBody");
    }

    #[tokio::test]
    async fn test_receive_block_in_slices_first_smaller_than_header() {
        let mut block_collector = BlockCollector {
            current_partial_block: Vec::new(),
            current_block_specified_length: None,
        };

        let mut block = DXBBlock::new_with_body(b"TestBody");
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::None);
        let block_bytes = block.to_bytes();

        let part1 = &block_bytes[0..2]; // smaller than header
        let part2 = &block_bytes[2..];

        assert!(block_collector.receive_slice(part1).await.is_none());

        assert_eq!(block_collector.current_partial_block.len(), part1.len());
        assert_eq!(block_collector.current_block_specified_length, None);

        let received_block = block_collector.receive_slice(part2).await;
        assert!(received_block.is_some());
        assert_eq!(received_block.unwrap().body, b"TestBody");
    }

    #[tokio::test]
    async fn test_receive_invalid_block() {
        let mut block_collector = BlockCollector {
            current_partial_block: Vec::new(),
            current_block_specified_length: None,
        };
        let invalid_block_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]; // Invalid magic number
        let received_block =
            block_collector.receive_slice(&invalid_block_bytes).await;
        assert!(received_block.is_none());
        assert!(block_collector.current_partial_block.is_empty());
        assert!(block_collector.current_block_specified_length.is_none());

        let mut block = DXBBlock::new_with_body(b"ValidBody");
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::None);

        let valid_block_bytes = block.to_bytes();
        let received_block =
            block_collector.receive_slice(&valid_block_bytes).await;
        assert!(received_block.is_some());
        assert_eq!(received_block.unwrap().body, b"ValidBody");
    }
}
