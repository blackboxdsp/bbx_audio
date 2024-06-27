use std::collections::HashMap;

use crate::{block::Block, sample::Sample};

/// A collection of interconnected `Block` objects.
pub struct Graph {
    sample_rate: usize,
    blocks: HashMap<usize, Block>,
    processes: HashMap<usize, Sample>,
    processing_order: Vec<usize>,
}

impl Graph {
    pub fn new(sample_rate: usize) -> Graph {
        return Graph {
            sample_rate,
            blocks: HashMap::new(),
            processes: HashMap::new(),
            processing_order: Vec::new(),
        };
    }

    pub fn sample_rate(&self) -> usize {
        return self.sample_rate;
    }
}

impl Graph {
    pub fn add_block(&mut self, block: Block) {
        let block_id = block.id;
        self.blocks.insert(block_id, block);
        self.processes.insert(block_id, 0.0);
    }

    pub fn create_connection(&self, source: &mut Block, destination: &mut Block) {
        source.add_output(destination.id);
        destination.add_input(source.id);
    }

    pub fn prepare_for_playback(&mut self) {
        self.update_processing_order();
    }
}

impl Graph {
    fn update_processing_order(&mut self) {
        let mut stack: Vec<usize> = Vec::with_capacity(self.blocks.len());
        let mut visited: Vec<usize> = Vec::with_capacity(self.blocks.len());

        fn dfs(block: &Block, order: &mut Vec<usize>, visited: &mut Vec<usize>, blocks: &HashMap<usize, Block>) {
            visited.push(block.id);
            for &block_id in &block.outputs {
                if visited.contains(&block_id) {
                    continue;
                } else {
                    let block_option = blocks.get(&block_id);
                    if let Some(block) = block_option {
                        dfs(block, order, visited, blocks);
                    }
                }
            }
            order.push(block.id);
        }

        for (_, block) in &self.blocks {
            if visited.contains(&block.id) {
                continue;
            } else {
                dfs(block, &mut stack, &mut visited, &self.blocks);
            }
        }

        if stack.len() == self.blocks.len() {
            self.processing_order = stack.clone();
            self.processing_order.reverse();
        } else {
            // TODO: Error
            println!("BAD\n{:#?}", stack);
        }
    }
}

impl Graph {
    pub fn evaluate(&mut self) -> Sample {
        for &block_id in &self.processing_order {
            let mut output_value: Sample = 0.0;
            let block_option = self.blocks.get_mut(&block_id);
            if let Some(block) = block_option {
                let num_inputs = block.inputs.len();
                if num_inputs > 0 {
                    let mut input_value: Sample = 0.0;
                    for input in &block.inputs {
                        if let Some(&single_input_value) = self.processes.get(input) {
                            input_value += single_input_value;
                        } else {
                            input_value += 0.0;
                        }
                    }
                    input_value /= num_inputs as f32;
                    output_value = block.operation.process(Some(input_value));
                } else {
                    output_value = block.operation.process(None);
                }
            } else {
                // Error - cannot retrieve current block
            }
            self.processes.insert(block_id, output_value);
        }

        let last_process_option = self.processes.get(self.processing_order.last().unwrap());
        return if let Some(&last_process_value) = last_process_option {
            last_process_value
        } else {
            0.0
        };
    }
}
