use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct FlexpoolBalance {
    error: serde_json::Value,
    result: u64,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct EthermineData {
    time: serde_json::Value,
    lastSeen: serde_json::Value,
    reportedHashrate: serde_json::Value,
    currentHashrate: serde_json::Value,
    validShares: serde_json::Value,
    invalidShares: serde_json::Value,
    staleShares: serde_json::Value,
    averageHashrate: serde_json::Value,
    activeWorkers: serde_json::Value,
    unpaid: u64,
    unconfirmed: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct EthermineBalance {
    status: serde_json::Value,
    data: EthermineData,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Eth2MinersStats {
    balance: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Eth2MinersBalance {
    #[serde(rename = "24hnumreward")]
    numreward: serde_json::Value,
    #[serde(rename = "24hreward")]
    reward: serde_json::Value,
    currentHashrate: serde_json::Value,
    currentLuck: serde_json::Value,
    hashrate: serde_json::Value,
    minerCharts: serde_json::Value,
    pageSize: serde_json::Value,
    payments: serde_json::Value,
    paymentsTotal: serde_json::Value,
    rewards: serde_json::Value,
    roundShares: serde_json::Value,
    stats: Eth2MinersStats,
}

#[derive(Deserialize, Debug)]
struct F2PoolBalance {
    balance: f64,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct HiveonBalance {
    earningStats: serde_json::Value,
    expectedReward24H: serde_json::Value,
    expectedRewardWeek: serde_json::Value,
    pendingPayouts: serde_json::Value,
    succeedPayouts: serde_json::Value,
    totalUnpaid: f64,
}

fn flexpool(wallet: &str, hashrate: f64) -> Result<f64, ureq::Error> {
    let url: String = format!("https://flexpool.io/api/v1/miner/{}/balance/", wallet);

    let jresponse: FlexpoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(((jresponse.result as f64) / 1000000000000000000.0) * (100.0 as f64) / hashrate)
}

fn ethermine(wallet: &str, hashrate: f64) -> Result<f64, ureq::Error> {
    let url: String = format!("https://api.ethermine.org/miner/{}/currentStats", wallet);

    let jresponse: EthermineBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(((jresponse.data.unpaid as f64) / 1000000000000000000.0) * (100.0 as f64) / hashrate)
}

fn eth2miners(wallet: &str, hashrate: f64) -> Result<f64, ureq::Error> {
    let url: String = format!("https://eth.2miners.com/api/accounts/{}", wallet);

    let jresponse: Eth2MinersBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(((jresponse.stats.balance as f64) / 1000000000.0) * (100.0 as f64) / hashrate)
}

fn f2pool(wallet: &str, hashrate: f64) -> Result<f64, ureq::Error> {
    let url: String = format!("https://api.f2pool.com/eth/{}", wallet);

    let jresponse: F2PoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok((jresponse.balance) * (100.0 as f64) / hashrate)
}

fn hiveon(wallet: &str, hashrate: f64) -> Result<f64, ureq::Error> {
    let url: String = format!(
        "https://hiveon.net/api/v1/stats/miner/{}/ETH/billing-acc",
        wallet
            .char_indices()
            .nth(2)
            .and_then(|(i, _)| wallet.get(i..))
            .unwrap_or("")
    );

    let jresponse: HiveonBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok((jresponse.totalUnpaid) * (100.0 as f64) / hashrate)
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct PoolConfig {
    check: bool,
    wallet: String,
    hashrate: f64,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Pools {
    flexpool: PoolConfig,
    ethermine: PoolConfig,
    #[serde(rename = "2miners")]
    eth2miners: PoolConfig,
    f2pool: PoolConfig,
    hiveon: PoolConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Config {
    pools: Pools,
}

fn read(var: &mut String, pool: &str, cfg: &mut PoolConfig) {
    use std::io;
    use std::io::Write;
    
    // check if active
    print!("{}? [Y/n]: ", pool);
    io::stdout().flush().unwrap();
    std::io::stdin().read_line(var).expect("Failed");
    if var.to_lowercase().contains("y") {
        cfg.check = true;
    } else {
        return;
    }
    // wallet address
    println!("Provide the wallet address:");
    let mut wallet = String::new();
    std::io::stdin().read_line(&mut wallet).expect("Failed");
    cfg.wallet = wallet.trim().to_string();

    // wallet address
    println!("Provide the reported hashrate:");
    let mut hashrate = String::new();
    std::io::stdin().read_line(&mut hashrate).expect("Failed");
    cfg.hashrate = hashrate.trim().to_string().parse::<f64>().unwrap_or(1.0);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config;
    if std::path::Path::new("config.yml").exists() {
        config = serde_yaml::from_reader(std::fs::File::open("config.yml")?)?;
    } else {
        config = Config::default();
        let mut flexpool = String::new();
        read(&mut flexpool, "Flexpool", &mut config.pools.flexpool);
        let mut ethermine = String::new();
        read(&mut ethermine, "Ethermine", &mut config.pools.ethermine);
        let mut eth2miners = String::new();
        read(&mut eth2miners, "2miners", &mut config.pools.eth2miners);
        let mut f2pool = String::new();
        read(&mut f2pool, "F2Pool", &mut config.pools.f2pool);
        let mut hiveon = String::new();
        read(&mut hiveon, "Hiveon", &mut config.pools.hiveon);

        use std::fs;
        fs::write("config.yml", serde_yaml::to_string(&config)?).expect("Unable to write file");
    }

    #[derive(Debug)]
    struct Pool {
        name: String,
        balance: f64,
    }

    let mut pools: Vec<Pool> = vec![];

    if config.pools.flexpool.check {
        pools.push(Pool {
            name: "Flexpool".to_string(),
            balance: flexpool(&config.pools.flexpool.wallet, 367.7).unwrap_or(0.0),
        });
    }
    if config.pools.ethermine.check {
        pools.push(Pool {
            name: "Ethermine".to_string(),
            balance: ethermine(&config.pools.ethermine.wallet, 366.5).unwrap_or(0.0),
        });
    }
    if config.pools.eth2miners.check {
        pools.push(Pool {
            name: "2miners".to_string(),
            balance: eth2miners(&config.pools.eth2miners.wallet, 367.0).unwrap_or(0.0),
        });
    }
    if config.pools.f2pool.check {
        pools.push(Pool {
            name: "F2Pool".to_string(),
            balance: f2pool(&config.pools.f2pool.wallet, 370.46).unwrap_or(0.0),
        });
    }
    if config.pools.hiveon.check {
        pools.push(Pool {
            name: "Hiveon".to_string(),
            balance: hiveon(&config.pools.hiveon.wallet, 357.52).unwrap_or(0.0),
        });
    }

    pools.sort_by(|a, b| (b.balance).partial_cmp(&a.balance).unwrap());

    for pool in pools.iter() {
        print!("{}: {} ETH", pool.name, pool.balance);
        if pool.name == pools[0].name {
            println!();
        } else {
            println!(" ({:.2}%)", pool.balance * 100.0 / pools[0].balance - 100.0)
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::thread;
        thread::park();
    }

    Ok(())
}
