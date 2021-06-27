use proc_macro::*;
use quote::quote;
use syn::{parse_macro_input, Ident, LitStr};
use syn::{ItemFn, ItemStruct};

pub(crate) fn process_tokens(input: String) -> String {
    let tokens = get_tokens(&input);
    let enum_generated = enum_generator(&tokens);
    let enum_impl_generated = enum_impl_generator(&tokens);
    let trait_generated = trait_generator(&tokens);
    let trait_impl_generated = trait_impl_generator(&tokens);
    [
        enum_generated,
        enum_impl_generated,
        trait_generated,
        trait_impl_generated,
    ]
    .join("\n")
}
#[derive(Debug)]
struct Tokens {
    name: String,
    params: Vec<String>,
    ret: Option<String>,
}

enum TokenProcess {
    None,
    Name,
    Params,
    Ret,
}

fn get_tokens(input: &str) -> Tokens {
    let mut name = String::from("");
    let mut params = vec![];
    let mut r = String::from("");

    let mut process = TokenProcess::Name;
    let mut move_next_param = true;

    input.chars().for_each(|s| {
        match process {
            TokenProcess::Name => {
                if s.is_whitespace() {
                    return;
                }

                if s == '<' {
                    process = TokenProcess::Params;
                } else if s == '-' {
                    process = TokenProcess::Ret;
                } else {
                    name.push(s);
                }
            }
            TokenProcess::Params => {
                if s == '>' {
                    process = TokenProcess::None;
                } else if s == ',' {
                    move_next_param = true;
                } else {
                    if move_next_param {
                        move_next_param = false;
                        params.push(String::from(""));
                    }

                    if let Some(param) = params.last_mut() {
                        param.push(s);
                    }
                }
            }
            TokenProcess::None => {
                if s.is_whitespace() {
                    return;
                }

                if s == '-' {
                    process = TokenProcess::Ret;
                }
            }
            TokenProcess::Ret => {
                if s == '>' {
                    return;
                }
                r.push(s);
            }
        };
    });

    Tokens {
        name,
        params,
        ret: if r == "" { None } else { Some(r) },
    }
}

fn enum_generator(tokens: &Tokens) -> String {
    let callback_ident = format!("{}Callback", tokens.name);
    format!(
        "pub enum {}<S> {{ {} }}",
        callback_ident,
        enum_generics(&combo(&tokens.params), tokens.ret.as_ref())
    )
}

fn enum_impl_generator(tokens: &Tokens) -> String {
    let callback_ident = format!("{}Callback", tokens.name);
    let params = params_generics(&tokens.params);
    let ret = tokens
        .ret
        .as_ref()
        .map(|v| format!(" -> {}", v))
        .unwrap_or("".to_string());
    let callback = enum_callback_generics(&combo(&tokens.params), &tokens.params);

    format!(
        r#"impl<S> {callback_ident}<S> {{
            pub(crate) fn exec(&self, {params}){ret} {{
                use {callback_ident}::*;
                match self {{
                   {callback}
                }}
            }}
        }}"#,
        callback_ident = callback_ident,
        params = params,
        ret = ret,
        callback = callback,
    )
}

fn trait_generator(tokens: &Tokens) -> String {
    let callback_ident = format!("{}Callback", tokens.name);
    let handler_ident = format!("{}Handler", tokens.name);
    format!(
        r#"
    pub trait {}<S, Params> {{
        fn callback(self) -> {}<S>;
    }}
    "#,
        handler_ident, callback_ident
    )
}

fn trait_impl_generator(tokens: &Tokens) -> String {
    let callback_ident = format!("{}Callback", tokens.name);
    let handler_ident = format!("{}Handler", tokens.name);
    let combinations = combo(&tokens.params);
    let ret = tokens
        .ret
        .as_ref()
        .map(|v| format!(" -> {}", v))
        .unwrap_or("".to_string());

    combinations
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let params = n.join(", ");

            format!(
                r#"
        #[allow(unused_parens)]
        impl<F, S> {handler_ident}<S, ({params})> for F
        where
            F: Fn({params}){ret} + 'static,
            S: AppState
        {{
            fn callback(self) -> {callback_ident}<S> {{
                {callback_ident}::_{i}(Box::new(self))
            }}
        }}
    "#,
                callback_ident = callback_ident,
                handler_ident = handler_ident,
                params = params,
                ret = ret,
                i = i
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn combo(arr: &[String]) -> Vec<Vec<String>> {
    let mut combi: Vec<Vec<String>> = vec![vec![String::from("")]];
    let mut temp: Vec<String> = vec![];
    let nn = num::pow::pow(0, 0);
    let slent = num::pow::pow(2, arr.len());

    for i in 0..slent {
        temp = vec![];
        for j in 0..arr.len() {
            if (i & num::pow::pow(2, j)) != 0 {
                temp.push(arr[j].clone());
            }
        }

        if temp.len() > 0 {
            combi.push(temp);
        }
    }

    combi
}

fn enum_generics(g: &[Vec<String>], r: Option<&String>) -> String {
    g.iter()
        .enumerate()
        .map(|(i, n)| {
            let gen = n.join(", ");
            format!(
                "_{}(Box<dyn Fn({}){}>)",
                i,
                gen,
                r.map(|v| format!(" -> {}", v)).unwrap_or("".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn params_generics(g: &[String]) -> String {
    g.iter()
        .enumerate()
        .map(|(i, n)| format!("param_{}: {}", i, n))
        .collect::<Vec<_>>()
        .join(",")
}

fn enum_callback_generics(g: &[Vec<String>], list: &[String]) -> String {
    g.iter()
        .enumerate()
        .map(|(i, n)| {
            let gen = n
                .iter()
                .map(|gen| {
                    if gen.is_empty() {
                        String::from("")
                    } else {
                        let index = list.iter().position(|g| g == gen).unwrap();
                        format!("param_{}", index)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("_{}(cb) => cb({})", i, gen)
        })
        .collect::<Vec<_>>()
        .join(",")
}