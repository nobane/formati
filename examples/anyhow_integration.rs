// Run with: cargo run --example anyhow_integration --features anyhow

#[cfg(feature = "anyhow")]
use anyhow::Result;
#[cfg(feature = "anyhow")]
use formati::{anyhow, bail};

#[cfg(feature = "anyhow")]
#[derive(Debug)]
struct Account {
    id: u32,
    balance: f64,
    owner: String,
}

#[cfg(feature = "anyhow")]
fn withdraw(account: &mut Account, amount: f64) -> Result<()> {
    if amount <= 0.0 {
        bail!("Invalid withdrawal amount {amount} for account {account.id}");
    }

    if account.balance < amount {
        bail!("Insufficient funds: {account.balance} < {amount} for {account.owner}");
    }

    account.balance -= amount;
    Ok(())
}

#[cfg(feature = "anyhow")]
fn main() -> Result<()> {
    let mut account = Account {
        id: 12345,
        balance: 100.0,
        owner: "Alice".to_string(),
    };

    // This will fail
    if let Err(e) = withdraw(&mut account, 150.0) {
        println!("Error: {e}");
    }

    // Create error directly
    let err = anyhow!("Account {account.id} belongs to {account.owner}");
    println!("Custom error: {err}");

    Ok(())
}

#[cfg(not(feature = "anyhow"))]
fn main() {
    println!(
        "This example requires the 'anyhow' feature. Run with: cargo run --example anyhow_integration --features anyhow"
    );
}
