use crate::Compile;
use crate::parse::case::Case;
use proc_macro2::TokenStream;
use quote::quote;

pub struct CaseActionGenerator {
    case: Case,
    input_var: TokenStream,
}

impl CaseActionGenerator {
    pub fn new(case: Case, input_var: TokenStream) -> Self {
        Self { case, input_var }
    }

    fn pattern_match_code(&self) -> TokenStream {
        self.case.pattern.generate_full_pattern()
    }

    fn input_unpacking_code(&self) -> TokenStream {
        let indices = 0..self.case.pattern.len();
        let input_var = &self.input_var;
        quote! {
            #(#input_var[#indices]),*
        }
    }
}

impl Compile for CaseActionGenerator {
    fn generate(self) -> TokenStream {
        let pattern = self.pattern_match_code();
        let body = &self.case.body;
        let unpacking = self.input_unpacking_code();

        quote! {
            match (#unpacking) {
                (#pattern) => {
                    #body
                },
                _ => panic!("not good")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::pattern::Pattern;
    use crate::utils::compare_token_streams;
    use quote::quote;

    #[test]
    fn test_input_unpacking_code() {
        let generator = CaseActionGenerator::new(
            Case {
                pattern: Pattern::parse(quote!(A && B && C)).unwrap(),
                guard: quote!(),
                body: quote!(),
            },
            quote!(input),
        );

        let output = generator.input_unpacking_code();
        let expected = quote!(input[0usize], input[1usize], input[2usize]);

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_pattern_match_code() {
        let generator = CaseActionGenerator::new(
            Case {
                pattern: Pattern::parse(quote!(A(a, b) && B(_, c) && C(d))).unwrap(),
                guard: quote!(),
                body: quote!(),
            },
            quote!(input),
        );

        let output = generator.pattern_match_code();
        let expected = quote!(A(a, b), B(_, c), C(d));

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_action_code() {
        let generator = CaseActionGenerator::new(
            Case {
                pattern: Pattern::parse(quote!(A(a, b) && B(_, c) && C(d))).unwrap(),
                guard: quote!(),
                body: quote! {
                    println!("Success");
                },
            },
            quote!(input),
        );

        let output = generator.generate();

        let expected = quote! {
            match (input[0usize], input[1usize], input[2usize]) {
                (A(a, b), B(_, c), C(d)) => {
                    println!("Success");
                },
                _ => panic!("not good")
            }
        };

        compare_token_streams(&expected, &output);
    }
}
