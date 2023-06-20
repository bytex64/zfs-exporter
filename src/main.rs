use std::collections::BTreeMap;
use std::thread;
use std::time;

use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
#[macro_use]
extern crate rocket;
use libzetta::zfs::{Properties, ZfsEngine, ZfsOpen3};
use libzetta::zpool::{open3::StatusOptions, ZpoolEngine, ZpoolOpen3};
use rocket::State;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct Labels {
    pool: String,
    dataset: String,
}

type FsMetric = Family<Labels, Gauge>;

struct Data {
    used: FsMetric,
    available: FsMetric,
}

#[derive(Responder)]
#[response(
    status = 200,
    content_type = "application/openmetrics-text; version=1.0.0; charset=utf-8"
)]
struct OpenMetrics(String);

#[get("/metrics")]
fn metrics(registry: &State<Registry>) -> OpenMetrics {
    let mut encoded = String::new();
    encode(&mut encoded, registry).expect("encoded");

    OpenMetrics(encoded)
}

fn init_registry(registry: &mut Registry) -> BTreeMap<String, Data> {
    let mut fsdata = BTreeMap::new();
    let zpool = ZpoolOpen3::default();
    let zfs = ZfsOpen3::new();

    let pools = zpool
        .status_all(StatusOptions::default())
        .expect("zpool status_all");
    for p in pools {
        let pool = p.name();
        let filesystems = zfs.list_filesystems(pool).expect("zfs");
        for fs in filesystems {
            let dataset = fs.display().to_string();

            let used = Family::<Labels, Gauge>::default();
            registry.register(
                "zfs_dataset_space_used",
                "space used by the dataset",
                used.clone(),
            );

            let available = Family::<Labels, Gauge>::default();
            registry.register(
                "zfs_dataset_space_available",
                "space available to the dataset",
                available.clone(),
            );
            fsdata.insert(dataset, Data { used, available });
        }
    }

    fsdata
}

fn collect_data(fsdata: &BTreeMap<String, Data>) {
    let zpool = ZpoolOpen3::default();
    let zfs = ZfsOpen3::new();

    let pools = zpool
        .status_all(StatusOptions::default())
        .expect("zpool status_all");
    for p in pools {
        let pool = p.name();
        let filesystems = zfs.list_filesystems(pool).expect("zfs");
        for fs in filesystems {
            let dataset = fs.display().to_string();
            let properties = zfs.read_properties(fs).expect("zfs properties");
            let filesystem_properties = match properties {
                Properties::Filesystem(fsp) => fsp,
                _ => unreachable!(),
            };
            let data = fsdata.get(&dataset).expect("fsdata get");
            let labels = Labels { pool: pool.to_owned(), dataset };
            data.used
                .get_or_create(&labels)
                .set(*filesystem_properties.used_by_dataset() as i64);
            data.available
                .get_or_create(&labels)
                .set(*filesystem_properties.available());
        }
    }
}

#[launch]
fn rocket() -> _ {
    let mut registry = <Registry>::default();

    let fsdata = init_registry(&mut registry);

    thread::spawn(move || {
        loop {
            collect_data(&fsdata);
            thread::sleep(time::Duration::from_secs(60));
        }
    });

    rocket::build()
        .manage(registry)
        .mount("/", routes![metrics])
}
