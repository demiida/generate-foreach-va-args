use std::fs::File;
use std::io::Write;
use std::process::exit;

use clap::Parser;

const DEFAULT_OUTPUT_FILE: &'static str = "foreach-va-args.h";

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    // 出力テキストファイルの名前
    #[arg(short, default_value = DEFAULT_OUTPUT_FILE)]
    output_file: String,

    // 列挙型の基底整数型のビット数
    #[arg(short, default_value_t = 8)]
    number_of_storage_type_bits: u8,
}

fn main() {
    let valid_ns: Vec<u8> = vec![8, 16];

    let args: Args = Args::parse();
    let n = match args.number_of_storage_type_bits {
        n if valid_ns.contains(&n) => n,
        n => exit_because_of_invalid_parameter("NUMBER_OF_STORAGE_TYPE_BITS", &valid_ns, n),
    };

    let text: Vec<u8> = generate_text(n).unwrap_or_else(|error| print_info_and_exit(error));

    File::create(args.output_file)
        .unwrap_or_else(|error| print_info_and_exit(error))
        .write_all(&text)
        .unwrap_or_else(|error| print_info_and_exit(error));
}

fn generate_text(n: u8) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let max_number_of_types: u32 = 1 << n;

    write!(
        &mut buffer,
        "\
// 参考にしました : https://in-neuro.hatenablog.com/entry/2020/10/21/155651
// {max_number_of_types} 個以下の可変長引数のマクロで使える foreach

#pragma once

#define CONCATENATE_AUX(x, y) x##y
#define CONCATENATE(x, y) CONCATENATE_AUX(x, y)

#define REVERSED_SEQ() {}
#define VA_ARGS_SIZE_IMPL({}, N, ...) N
#define VA_ARGS_SIZE_AUX(...) VA_ARGS_SIZE_IMPL(__VA_ARGS__)
#define VA_ARGS_SIZE(...) VA_ARGS_SIZE_AUX(__VA_ARGS__, REVERSED_SEQ())

#define ADD_PREFIX_FOREACH_VA_ARGS_AUX_0(prefix, ARG)
",
        (0..=max_number_of_types)
            .rev()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(", "),
        (0..max_number_of_types)
            .map(|i| format!("ARG{i}"))
            .collect::<Vec<String>>()
            .join(", ")
    )?;

    for i in 1..=max_number_of_types {
        write!(
            &mut buffer,
            "\
#define ADD_PREFIX_FOREACH_VA_ARGS_AUX_{}(prefix, ARG, ...) \\
    prefix##ARG, ADD_PREFIX_FOREACH_VA_ARGS_AUX_{}(prefix, __VA_ARGS__)
",
            i,
            i - 1
        )?;
    }

    write!(
        &mut buffer,
        "\
#define ADD_PREFIX_FOREACH_VA_ARGS(prefix, ...) \\
    CONCATENATE(ADD_PREFIX_FOREACH_VA_ARGS_AUX_, VA_ARGS_SIZE(__VA_ARGS__))(prefix, __VA_ARGS__)

#define FOREACH_VA_ARGS_AUX_0(FUNCTOR, ARG)
"
    )?;

    for i in 1..=max_number_of_types {
        write!(
            &mut buffer,
            "\
#define FOREACH_VA_ARGS_AUX_{}(FUNCTOR, ARG, ...) \\
    FUNCTOR(ARG) FOREACH_VA_ARGS_AUX_{}(FUNCTOR, __VA_ARGS__)
",
            i,
            i - 1
        )?;
    }

    write!(
        &mut buffer,
        "\
#define FOREACH_VA_ARGS(FUNCTOR, ...) \\
    CONCATENATE(FOREACH_VA_ARGS_AUX_, VA_ARGS_SIZE(__VA_ARGS__))(FUNCTOR, __VA_ARGS__)
"
    )?;

    Ok(buffer)
}

fn print_info_and_exit<T>(info: T) -> !
where
    T: std::fmt::Display,
{
    println!("{info}");
    exit(-1);
}

fn exit_because_of_invalid_parameter<T>(
    parameter_name: &str,
    valid_parameters: &Vec<T>,
    invalid_parameter: T,
) -> !
where
    T: std::fmt::Display,
{
    let valid_parameters = valid_parameters
        .iter()
        .map(|n| format!("{n}"))
        .reduce(|left, right| format!("{left}, {right}"))
        .unwrap_or(String::new());

    println!("{parameter_name} に無効な値 {invalid_parameter} が設定されました。");
    println!("有効な値は {valid_parameters} です。");
    exit(-1);
}
