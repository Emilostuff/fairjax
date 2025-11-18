use crate::traits::CaseBundle;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;

/// Generates element mappings that define subpattern positions for permutation algorithms
pub trait MappingCodeGen {
    /// Generate code creating Element instances for each unique subpattern in the pattern
    fn generate(span: Span, bundle: &dyn CaseBundle, ident: &Ident) -> TokenStream;
}

pub struct MappingCompiler;

impl MappingCodeGen for MappingCompiler {
    fn generate(span: Span, bundle: &dyn CaseBundle, ident: &Ident) -> TokenStream {
        // Generate Element instances containing position lists for each subpattern
        let inputs: Vec<Vec<usize>> = bundle
            .sub_pattern_groups()
            .0
            .iter()
            .flat_map(|sp_stats| {
                (0..sp_stats.size())
                    .map(|_| sp_stats.occurrences().clone())
                    .collect::<Vec<Vec<usize>>>()
            })
            .collect();

        let pattern_size = inputs.len();

        let elements: Vec<_> = inputs.into_iter().enumerate().collect();
        let mut mappings = Vec::new();
        let mut current = vec![0; pattern_size];
        let mut used = vec![false; pattern_size];

        Self::permute(&elements, &mut mappings, &mut current, &mut used);

        let mappings_code = mappings
            .into_iter()
            .map(|mapping| {
                quote_spanned! { span =>
                    fairjax_core::mapping::Mapping::<#pattern_size> ( [ #(#mapping),*] )
                }
            })
            .collect::<Vec<_>>();

        // Assemble element mappings
        let len = mappings_code.len();
        quote_spanned!( span => const #ident: [fairjax_core::mapping::Mapping::<#pattern_size>; #len] = [ #(#mappings_code),* ]; )
    }
}

impl MappingCompiler {
    fn permute(
        elements: &[(usize, Vec<usize>)],
        output: &mut Vec<Vec<usize>>,
        current: &mut Vec<usize>,
        used: &mut Vec<bool>,
    ) {
        if let Some((origin, element)) = elements.first() {
            for &i in element.iter() {
                if !used[i] {
                    used[i] = true;
                    (*current)[i] = *origin;
                    MappingCompiler::permute(&elements[1..], output, current, used);
                    used[i] = false;
                }
            }
        } else {
            // Push valid permutation to output
            output.push(current.clone());
        }
    }
}
