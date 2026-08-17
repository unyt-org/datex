use crate::{
    global::protocol_structures::{
        instruction_data::StackIndex,
        instructions::{Instruction, NextExpectedInstructions},
        regular_instructions::RegularInstruction,
        type_instructions::TypeInstruction,
    },
    prelude::*,
};

pub trait CollectionResultsPopper<
    Result,
    Val,
    Key,
    KeyVal,
    Type,
    TypeDefinition,
>: GetResults<Result> + Sized
{
    fn try_extract_value(result: Result) -> Option<Val>;
    fn try_extract_type(result: Result) -> Option<Type>;
    fn try_extract_type_definition(result: Result) -> Option<TypeDefinition>;
    fn try_extract_key_value_pair(result: Result) -> Option<(Key, KeyVal)>;

    fn try_pop_value(&mut self) -> Option<Val> {
        let result = self.pop()?;
        Self::try_extract_value(result)
    }
    fn try_pop_type(&mut self) -> Option<Type> {
        let result = self.pop()?;
        Self::try_extract_type(result)
    }
    fn try_pop_type_definition(&mut self) -> Option<TypeDefinition> {
        let result = self.pop()?;
        Self::try_extract_type_definition(result)
    }
    fn try_pop_key_value_pair(&mut self) -> Option<(Key, KeyVal)> {
        let result = self.pop()?;
        Self::try_extract_key_value_pair(result)
    }
    fn pop_values(&mut self, count: u32) -> Vec<Val> {
        let mut values = Vec::with_capacity(count as usize);
        for _ in 0..count {
            values.push(self.pop_value());
        }
        values.reverse();
        values
    }

    fn pop_value(&mut self) -> Val {
        self.try_pop_value().expect("Expected value result")
    }
    fn pop_type(&mut self) -> Type {
        self.try_pop_type()
            .expect("Expected type definition result")
    }
    fn pop_type_definition(&mut self) -> TypeDefinition {
        self.try_pop_type_definition()
            .expect("Expected type definition result")
    }
    fn pop_key_value_pair(&mut self) -> (Key, KeyVal) {
        self.try_pop_key_value_pair()
            .expect("Expected key-value pair result")
    }

    fn pop(&mut self) -> Option<Result> {
        self.get_results_mut().pop()
    }

    fn push(&mut self, result: Result) {
        self.get_results_mut().push(result);
    }

    fn len(&self) -> usize {
        self.get_results().len()
    }

    fn is_empty(&self) -> bool {
        self.get_results().is_empty()
    }

    /// Collects all value results
    /// Panics if any of the popped results are not value results
    fn collect_value_results(mut self) -> Vec<Val> {
        let count = self.len();
        let mut expressions = Vec::with_capacity(count);
        for _ in 0..count {
            expressions.push(self.pop_value());
        }
        expressions.reverse();
        expressions
    }

    /// Collects all key-value pair results
    /// Panics if any of the popped results are not key-value pairs
    fn collect_key_value_pair_results(mut self) -> Vec<(Key, KeyVal)> {
        let count = self.len();
        let mut expression_pairs = Vec::with_capacity(count);
        for _ in 0..count {
            let pair = self.pop_key_value_pair();
            expression_pairs.push(pair);
        }
        expression_pairs.reverse();
        expression_pairs
    }

    /// Collects all type results
    /// Panics if any of the popped results are not type results
    fn collect_type_results(mut self) -> Vec<Type> {
        let count = self.len();
        let mut type_expressions = Vec::with_capacity(count);
        for _ in 0..count {
            type_expressions.push(self.pop_type());
        }
        type_expressions.reverse();
        type_expressions
    }
}

#[derive(Debug)]
pub struct CollectedResults<T> {
    results: Vec<T>,
}

impl<T> Default for CollectedResults<T> {
    fn default() -> Self {
        CollectedResults {
            results: Vec::new(),
        }
    }
}

pub trait GetResults<T> {
    fn get_results(&self) -> &Vec<T>;
    fn get_results_mut(&mut self) -> &mut Vec<T>;
}

impl<T> GetResults<T> for CollectedResults<T> {
    fn get_results(&self) -> &Vec<T> {
        &self.results
    }
    fn get_results_mut(&mut self) -> &mut Vec<T> {
        &mut self.results
    }
}

#[derive(Debug)]
pub enum ResultCollector<T> {
    Full(FullResultCollector<T>),
    Last(LastResultCollector<T>),
    FullUnbounded(FullUnboundedResultCollector<T>),
    LastUnbounded(LastUnboundedResultCollector<T>),
}

pub enum FullOrPartialResult<T> {
    Full {
        instruction: Instruction,
        results: CollectedResults<T>,
    },
    Partial {
        instruction: Instruction,
        result: Option<T>,
        /// The stack index (size) before entering the instruction (normally Statements instruction) that
        /// produced this result. This is used to correctly clean up the stack at the end of a statements block.
        previous_stack_index: StackIndex,
    },
}

#[derive(Debug)]
pub struct FullResultCollector<T> {
    instruction: Option<Instruction>,
    expected_count: u32,
    collected_results: CollectedResults<T>,
}

#[derive(Debug)]
pub struct LastResultCollector<T> {
    instruction: Option<Instruction>,
    expected_count: u32,
    collected_count: u32,
    last_result: Option<T>,
    /// The stack index (size) before entering the instruction (normally Statements instruction) that
    /// produced this result. This is used to correctly clean up the stack at the end of a statements block.
    previous_stack_index: StackIndex,
}

#[derive(Debug)]
pub struct FullUnboundedResultCollector<T> {
    instruction: Option<Instruction>,
    collected_results: CollectedResults<T>,
}

#[derive(Debug)]
pub struct LastUnboundedResultCollector<T> {
    instruction: Option<Instruction>,
    pub(crate) last_result: Option<T>,
    /// The stack index (size) before entering the instruction (normally Statements instruction) that
    /// produced this result. This is used to correctly clean up the stack at the end of a statements block.
    previous_stack_index: StackIndex,
}

impl<T> ResultCollector<T> {
    pub fn skip(&mut self, count: u32) -> bool {
        match self {
            ResultCollector::Last(collector) => {
                if collector.collected_count.saturating_add(count)
                    > collector.expected_count
                {
                    panic!(
                        "Skipped more results than expected for the instruction"
                    );
                }
                collector.collected_count += count;
                true
            }
            _ => false,
        }
    }
    pub fn push_result(&mut self, result: impl Into<T>) {
        match self {
            ResultCollector::Full(collector) => {
                if collector.collected_results.get_results().len() as u32
                    >= collector.expected_count
                {
                    panic!(
                        "Collected more results than expected for the instruction"
                    );
                }
                collector
                    .collected_results
                    .get_results_mut()
                    .push(result.into());
            }
            ResultCollector::Last(collector) => {
                if collector.collected_count >= collector.expected_count {
                    panic!(
                        "Collected more results than expected for the instruction"
                    );
                }
                collector.last_result = Some(result.into());
                collector.collected_count += 1;
            }
            ResultCollector::FullUnbounded(collector) => {
                collector
                    .collected_results
                    .get_results_mut()
                    .push(result.into());
            }
            ResultCollector::LastUnbounded(collector) => {
                collector.last_result = Some(result.into());
            }
        }
    }

    pub fn try_pop_collected(&mut self) -> Option<FullOrPartialResult<T>> {
        match self {
            ResultCollector::Full(collector) => {
                if collector.collected_results.get_results().len() as u32
                    == collector.expected_count
                {
                    Some(FullOrPartialResult::Full {
                        instruction: collector.instruction.take().unwrap(),
                        results: core::mem::take(
                            &mut collector.collected_results,
                        ),
                    })
                } else if collector.collected_results.get_results().len() as u32
                    > collector.expected_count
                {
                    panic!(
                        "Collected more results than expected for the last instruction"
                    );
                } else {
                    None
                }
            }
            ResultCollector::Last(collector) => {
                if collector.collected_count == collector.expected_count {
                    Some(FullOrPartialResult::Partial {
                        instruction: collector.instruction.take().unwrap(),
                        result: collector.last_result.take(),
                        previous_stack_index: collector.previous_stack_index,
                    })
                } else if collector.collected_count > collector.expected_count {
                    panic!(
                        "Collected more results than expected for the last instruction"
                    );
                } else {
                    None
                }
            }
            // unbounded results must be explicitly popped with try_pop_unbounded
            ResultCollector::LastUnbounded(_) => None,
            ResultCollector::FullUnbounded(_) => None,
        }
    }

    pub fn try_pop_unbounded(&mut self) -> Option<FullOrPartialResult<T>> {
        match self {
            ResultCollector::LastUnbounded(collector) => {
                Some(FullOrPartialResult::Partial {
                    instruction: collector.instruction.take().unwrap(),
                    result: collector.last_result.take(),
                    previous_stack_index: collector.previous_stack_index,
                })
            }
            ResultCollector::FullUnbounded(collector) => {
                Some(FullOrPartialResult::Full {
                    instruction: collector.instruction.take().unwrap(),
                    results: core::mem::take(&mut collector.collected_results),
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct InstructionCollector<T> {
    result_collectors: Vec<ResultCollector<T>>,
    root_result: Option<T>,
}

impl<T> Default for InstructionCollector<T> {
    fn default() -> Self {
        InstructionCollector {
            result_collectors: Vec::new(),
            root_result: None,
        }
    }
}

#[derive(Debug)]
pub enum StatementResultCollectionStrategy {
    Full,
    Last,
}

impl<T> InstructionCollector<T> {
    pub fn skip_current_counted_results(&mut self, count: u32) {
        if count == 0 {
            return;
        }
        let collector = self
            .result_collectors
            .last_mut()
            .expect("Jump skipped instructions without an enclosing collector");
        assert!(
            collector.skip(count),
            "Jump skipped instructions in a non-statements collector"
        );
    }
    pub fn collect_full(
        &mut self,
        instruction: Instruction,
        expected_count: u32,
    ) {
        self.result_collectors.push(ResultCollector::Full(
            FullResultCollector {
                instruction: Some(instruction),
                expected_count,
                collected_results: CollectedResults::default(),
            },
        ));
    }

    pub fn collect_last(
        &mut self,
        instruction: Instruction,
        expected_count: u32,
        current_stack_index: StackIndex,
    ) {
        self.result_collectors.push(ResultCollector::Last(
            LastResultCollector {
                instruction: Some(instruction),
                expected_count,
                collected_count: 0,
                last_result: None,
                previous_stack_index: current_stack_index,
            },
        ));
    }

    pub fn collect_full_unbounded(&mut self, instruction: Instruction) {
        self.result_collectors.push(ResultCollector::FullUnbounded(
            FullUnboundedResultCollector {
                instruction: Some(instruction),
                collected_results: CollectedResults::default(),
            },
        ));
    }

    pub fn collect_last_unbounded(
        &mut self,
        instruction: Instruction,
        current_stack_index: StackIndex,
    ) {
        self.result_collectors.push(ResultCollector::LastUnbounded(
            LastUnboundedResultCollector {
                instruction: Some(instruction),
                last_result: None,
                previous_stack_index: current_stack_index,
            },
        ));
    }

    pub fn is_collecting(&self) -> bool {
        !self.result_collectors.is_empty()
    }

    pub fn push_result(&mut self, result: impl Into<T>) {
        let result = result.into();
        if let Some(result_collector) = self.result_collectors.last_mut() {
            result_collector.push_result(result);
        } else {
            self.root_result = Some(result);
        }
    }

    pub fn try_pop_collected(&mut self) -> Option<FullOrPartialResult<T>> {
        let result_collector = self.result_collectors.last_mut()?;
        let results = result_collector.try_pop_collected();
        if results.is_some() {
            self.result_collectors.pop();
        }
        results
    }

    pub fn try_pop_unbounded(&mut self) -> Option<FullOrPartialResult<T>> {
        let result_collector = self.result_collectors.last_mut()?;
        let results = result_collector.try_pop_unbounded();
        if results.is_some() {
            self.result_collectors.pop();
        }
        results
    }

    pub fn last(&self) -> Option<&ResultCollector<T>> {
        self.result_collectors.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut ResultCollector<T>> {
        self.result_collectors.last_mut()
    }

    pub fn take_root_result(&mut self) -> Option<T> {
        self.root_result.take()
    }

    /// Processes a regular instruction with default behavior for recursive instructions that need to
    /// collect more results.
    /// Returns Some(regular_instruction) if the instruction was not handled and should be processed by the caller.
    pub fn default_regular_instruction_collection(
        &mut self,
        regular_instruction: RegularInstruction,
        statement_result_collection_strategy: StatementResultCollectionStrategy,
        current_stack_index: StackIndex,
    ) -> Option<RegularInstruction> {
        let next_expected_instructions =
            regular_instruction.get_next_expected_instructions();

        match next_expected_instructions {
            NextExpectedInstructions::Regular(regular_count) => {
                // special case: statements if strategy is collect last
                if matches!(
                    statement_result_collection_strategy,
                    StatementResultCollectionStrategy::Last
                ) && matches!(
                    regular_instruction,
                    RegularInstruction::Statements(_)
                        | RegularInstruction::ShortStatements(_)
                ) {
                    self.collect_last(
                        Instruction::Regular(regular_instruction),
                        regular_count,
                        current_stack_index,
                    );
                }
                // normal collect
                else {
                    self.collect_full(
                        Instruction::Regular(regular_instruction),
                        regular_count,
                    );
                }
                None
            }
            NextExpectedInstructions::Type(type_count) => {
                self.collect_full(
                    Instruction::Regular(regular_instruction),
                    type_count,
                );
                None
            }
            NextExpectedInstructions::RegularAndType(
                regular_count,
                type_count,
            ) => {
                self.collect_full(
                    Instruction::Regular(regular_instruction),
                    regular_count + type_count,
                );
                None
            }
            NextExpectedInstructions::UnboundedStart => {
                match statement_result_collection_strategy {
                    StatementResultCollectionStrategy::Full => {
                        self.collect_full_unbounded(Instruction::Regular(
                            regular_instruction,
                        ));
                    }
                    StatementResultCollectionStrategy::Last => {
                        self.collect_last_unbounded(
                            Instruction::Regular(regular_instruction),
                            current_stack_index,
                        );
                    }
                }
                None
            }
            NextExpectedInstructions::UnboundedEnd => {
                self.collect_full(Instruction::Regular(regular_instruction), 0);
                None
            }

            NextExpectedInstructions::None => Some(regular_instruction),
        }
    }

    /// Processes a type instruction with default behavior for recursive instructions that need to
    /// collect more results.
    /// Returns Some(type_instruction) if the instruction was not handled and should be processed by the caller.
    pub fn default_type_instruction_collection(
        &mut self,
        type_instruction: TypeInstruction,
    ) -> Option<TypeInstruction> {
        let next_expected_instructions =
            type_instruction.get_next_expected_instructions();

        match next_expected_instructions {
            NextExpectedInstructions::Type(type_count) => {
                self.collect_full(
                    Instruction::Type(type_instruction),
                    type_count,
                );
                None
            }

            // currently not used for type instructions
            NextExpectedInstructions::Regular(_)
            | NextExpectedInstructions::RegularAndType(..)
            | NextExpectedInstructions::UnboundedStart
            | NextExpectedInstructions::UnboundedEnd => unreachable!(),

            NextExpectedInstructions::None => Some(type_instruction),
        }
    }
}
