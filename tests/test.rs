use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::predicate;
use std::process::Command;

#[test]
fn simple_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/simple_input.csv");
    cmd.assert().success().stdout(predicate::str::starts_with(
        "client,available,held,total,locked\n1,1.5,0,1.5,false\n2,2,0,2,false",
    ));

    Ok(())
}

#[test]
fn test_whitespace_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/white_space.csv");
    cmd.assert().success().stdout(predicate::str::starts_with(
        "client,available,held,total,locked\n1,1.5,0,1.5,false\n2,2,0,2,false",
    ));

    Ok(())
}

#[test]
fn test_all_types() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/all_types.csv");
    cmd.assert().success().stdout(predicate::str::starts_with(
        "client,available,held,total,locked\n1,-2,0,-2,true",
    ));

    Ok(())
}

#[test]
fn test_all_precisions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/all_precisions.csv");
    cmd.assert().success().stdout(predicate::str::starts_with(
        "client,available,held,total,locked\n1,1.1111,0,1.1111,false",
    ));

    Ok(())
}

#[test]
fn reject_transactions_on_frozen_clients() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/reject_on_frozen_client.csv");
    cmd.assert().success().stdout(predicate::str::starts_with(
        "client,available,held,total,locked\n5,0,0,0,true\n2,0,0,0,true\n4,0,0,0,true\n1,0,0,0,true\n3,0,0,0,true",
    ));

    Ok(())
}

#[test]
fn test_large_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("engine")?;
    cmd.arg("./files/large_test_file.csv");

    cmd.assert().success();

    Ok(())
}
