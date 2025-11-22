use test_prioritization::*;

fn main() -> anyhow::Result<()> {
    let text_content = get_test_case_file_text_content("Mozilla_TCs/TC1.html").unwrap();

    println!("{}", text_content);
    Ok(())
}
