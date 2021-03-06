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

#[derive(Deserialize, Debug)]
struct NanopoolBalance {
    status: serde_json::Value,
    data: f64,
}

#[derive(Deserialize, Debug)]
struct SparkPoolData {
    balance: f64,
}

#[derive(Deserialize, Debug)]
struct SparkPoolBalance {
    code: serde_json::Value,
    data: SparkPoolData,
}

fn flexpool(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!("https://flexpool.io/api/v1/miner/{}/balance/", wallet);

    let jresponse: FlexpoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok((jresponse.result as f64) / 1000000000000000000.0)
}

fn ethermine(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!("https://api.ethermine.org/miner/{}/currentStats", wallet);

    let jresponse: EthermineBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok((jresponse.data.unpaid as f64) / 1000000000000000000.0)
}

fn eth2miners(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!("https://eth.2miners.com/api/accounts/{}", wallet);

    let jresponse: Eth2MinersBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok((jresponse.stats.balance as f64) / 1000000000.0)
}

fn f2pool(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!("https://api.f2pool.com/eth/{}", wallet);

    let jresponse: F2PoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(jresponse.balance)
}

fn hiveon(wallet: &str) -> Result<f64, ureq::Error> {
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

    Ok(jresponse.totalUnpaid)
}

fn nanopool(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!("https://api.nanopool.org/v1/eth/balance/{}", wallet);

    let jresponse: NanopoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(jresponse.data)
}

fn sparkpool(wallet: &str) -> Result<f64, ureq::Error> {
    let url: String = format!(
        "https://www.sparkpool.com/v1/bill/stats?miner={}&currency=ETH",
        wallet
    );

    let jresponse: SparkPoolBalance = ureq::get(&url)
        .set("accept", "application/json")
        .call()?
        .into_json()?;

    Ok(jresponse.data.balance)
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct PoolConfig {
    check: bool,
    wallet: String,
    hashrate: f64,
    starting_balance: f64,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Pools {
    flexpool: PoolConfig,
    ethermine: PoolConfig,
    #[serde(rename = "2miners")]
    eth2miners: PoolConfig,
    f2pool: PoolConfig,
    hiveon: PoolConfig,
    nanopool: PoolConfig,
    sparkpool: PoolConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Config {
    pools: Pools,
}

fn read(pool: &str, cfg: &mut PoolConfig) {
    use std::io;
    use std::io::Write;

    // check if active
    let mut check_line: String = String::new();
    print!("{}? [Y/n]: ", pool);
    io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut check_line).expect("Failed");
    if check_line.to_lowercase().contains("y") {
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

    // starting balance
    print!("Subtract the current balance? [Y/n]: ");
    io::stdout().flush().unwrap();
    let mut line: String = String::new();
    std::io::stdin().read_line(&mut line).expect("Failed");
    if line.to_lowercase().contains("y") {
        cfg.starting_balance = match pool {
            "Flexpool" => flexpool(&cfg.wallet).unwrap_or(0.0),
            "Ethermine" => ethermine(&cfg.wallet).unwrap_or(0.0),
            "2miners" => eth2miners(&cfg.wallet).unwrap_or(0.0),
            "F2Pool" => f2pool(&cfg.wallet).unwrap_or(0.0),
            "Hiveon" => hiveon(&cfg.wallet).unwrap_or(0.0),
            "Nanopool" => nanopool(&cfg.wallet).unwrap_or(0.0),
            "SparkPool" => sparkpool(&cfg.wallet).unwrap_or(0.0),
            _ => {
                println!("Error!");
                0.0 as f64
            }
        }
    };
}

fn get_pool_config(name: &str) -> PoolConfig {
    let config: Config =
        serde_yaml::from_reader(std::fs::File::open("config.yml").unwrap()).unwrap(); // TODO: remove this

    match name {
        "Flexpool" => config.pools.flexpool,
        "Ethermine" => config.pools.ethermine,
        "2miners" => config.pools.eth2miners,
        "F2Pool" => config.pools.f2pool,
        "Hiveon" => config.pools.hiveon,
        "Nanopool" => config.pools.nanopool,
        "SparkPool" => config.pools.sparkpool,
        _ => {
            println!("Error!");
            return config.pools.ethermine;
        }
    }
}

#[derive(Debug)]
struct Pool {
    name: String,
    balance: f64,
}

fn get_pool_data(pool_name: &str, original_balance: f64, hashrate: f64) -> Pool {
    Pool {
        name: pool_name.to_string(),
        balance: original_balance * (100.0 as f64) / hashrate
            - get_pool_config(pool_name).starting_balance,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config;
    if std::path::Path::new("config.yml").exists() {
        config = serde_yaml::from_reader(std::fs::File::open("config.yml")?)?;
    } else {
        config = Config::default();
        read("Flexpool", &mut config.pools.flexpool);
        read("Ethermine", &mut config.pools.ethermine);
        read("2miners", &mut config.pools.eth2miners);
        read("F2Pool", &mut config.pools.f2pool);
        read("Hiveon", &mut config.pools.hiveon);
        read("Nanopool", &mut config.pools.nanopool);
        read("SparkPool", &mut config.pools.sparkpool);

        use std::fs;
        fs::write("config.yml", serde_yaml::to_string(&config)?).expect("Unable to write file");
    }

    let mut pools: Vec<Pool> = vec![];

    if config.pools.flexpool.check {
        pools.push(get_pool_data(
            "Flexpool",
            flexpool(&config.pools.flexpool.wallet).unwrap_or(0.0),
            config.pools.flexpool.hashrate,
        ));
    }
    if config.pools.ethermine.check {
        pools.push(get_pool_data(
            "Ethermine",
            ethermine(&config.pools.ethermine.wallet).unwrap_or(0.0),
            config.pools.ethermine.hashrate,
        ));
    }
    if config.pools.eth2miners.check {
        pools.push(get_pool_data(
            "2miners",
            eth2miners(&config.pools.eth2miners.wallet).unwrap_or(0.0),
            config.pools.eth2miners.hashrate,
        ));
    }
    if config.pools.f2pool.check {
        pools.push(get_pool_data(
            "F2Pool",
            f2pool(&config.pools.f2pool.wallet).unwrap_or(0.0),
            config.pools.f2pool.hashrate,
        ));
    }
    if config.pools.hiveon.check {
        pools.push(get_pool_data(
            "Hiveon",
            hiveon(&config.pools.hiveon.wallet).unwrap_or(0.0),
            config.pools.hiveon.hashrate,
        ));
    }
    if config.pools.nanopool.check {
        pools.push(get_pool_data(
            "Nanopool",
            nanopool(&config.pools.nanopool.wallet).unwrap_or(0.0),
            config.pools.nanopool.hashrate,
        ));
    }
    if config.pools.sparkpool.check {
        pools.push(get_pool_data(
            "SparkPool",
            sparkpool(&config.pools.sparkpool.wallet).unwrap_or(0.0),
            config.pools.sparkpool.hashrate,
        ));
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
