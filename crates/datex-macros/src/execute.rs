use std::str::FromStr;

use datex_core::{
    compiler::{CompileOptions, compile_template},
    runtime::execution::{
        ExecutionInput, ExecutionOptions, execute_dxb_sync,
        execution_loop::state::{RuntimeExecutionSlots, RuntimeExecutionState},
    },
    values::{
        core_values::integer::typed_integer::{
            IntegerTypeVariant, TypedInteger,
        },
        value_container::ValueContainer,
    },
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct ExecuteMacroInput {
    program: String,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for ExecuteMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let program: LitStr = input.parse()?;
        let mut args = Punctuated::<Expr, Token![,]>::new();

        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            while !input.is_empty() {
                args.push_value(input.parse()?);
                if input.peek(Token![,]) {
                    args.push_punct(input.parse()?);
                } else {
                    break;
                }
            }
        }

        Ok(Self {
            program: program.value(),
            args,
        })
    }
}

pub fn execute(input: ExecuteMacroInput) -> TokenStream {
    let script = input.program;
    let dxb = compile_template(&script, &vec![], CompileOptions::default());
    if let Err(e) = dxb {
        return syn::Error::new_spanned(
            script,
            format!("Failed to compile template: {}", e),
        )
        .to_compile_error();
    }
    let dxb = dxb.unwrap().0;

    let placeholder_count = script.chars().filter(|&c| c == '?').count();
    let arg_count = input.args.len();

    let inserts = input.args.iter().enumerate().map(|(i, expr)| {
        let idx = i as u32;
        quote! {
            slots.insert(#idx, Some(ValueContainer::from(#expr)));
        }
    });

    if placeholder_count != arg_count {
        return syn::Error::new_spanned(
            script,
            format!(
                "execute!: placeholder count ({}) != argument count ({})",
                placeholder_count, arg_count
            ),
        )
        .to_compile_error();
    }
    // let runtime_execution_slots = RuntimeExecutionSlots { slots };

    // let execution_state = RuntimeExecutionState {
    //     slots: runtime_execution_slots,
    //     runtime_internal: None,
    //     source_id: 0,
    // };

    quote! {{
        use datex_core::runtime::execution::execution_loop::state::{RuntimeExecutionState, RuntimeExecutionSlots};
        use datex_core::values::value_container::ValueContainer;
        use datex_core::collections::HashMap;
        use datex_core::runtime::execution::{execute_dxb_sync,ExecutionInput, ExecutionOptions};

        let mut slots: HashMap<u32, Option<ValueContainer>> = HashMap::new();
        #(#inserts)*
        let runtime_execution_slots = RuntimeExecutionSlots { slots };

        let execution_state = RuntimeExecutionState { slots: runtime_execution_slots, runtime_internal: None, source_id: 0 };
        let dxb_body = vec![#(#dxb),*];
        let execution_input = ExecutionInput::new(&dxb_body, ExecutionOptions::default(), None);
        execute_dxb_sync(execution_input)
        // execute_dxb_sync(
        //     ExecutionInput::new(vec![#(#dxb),*], loop_state, None).execution_loop()
        // )
    }}
}
