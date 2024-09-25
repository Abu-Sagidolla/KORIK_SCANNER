use std::sync::mpsc;
use std::thread;
use std::sync::Arc;
use sender::crawler::CrawlLink;
use sender::suraw::smuggle;
use futures::future::join_all;
use crate::sqli::sqli::sql_scanner;
use crate::xss::xss::XSSCAN;
use crate::sender::crawler;
use crate::sender::suraw;
use std::time::Duration;
use crate::method_parser::method_parser::get_methods;
use url::Url;
use crate::suraw::jiber;
use tokio::sync::Semaphore;


mod sqli;
mod xss;
mod sender;
mod method_parser;

#[derive(Debug,Clone)]

pub enum CUSTOMER {
    URL(String),
    DOMAIN(String),
    IP(String),
}

#[derive(Clone)]
struct SCANER {
    id: i32,
    options: OPTIONS,
    customer: CUSTOMER,
    speed: RATE,
}

impl SCANER {
    fn new(id: i32, options: OPTIONS, customer: CUSTOMER, speed: RATE) -> Self {
        SCANER {
            id,
            options,
            customer,
            speed,
        }
    }

    async fn start_scan(self: Arc<Self> ) {
        let mut tasks = vec![];

        match &self.options {
            OPTIONS::FULLSCAN(rate) => {
                let modules = vec![
                    MODULES::HOSTINGER,
                    MODULES::SQLI(SQL::UNION),
                    MODULES::SQLI(SQL::BLIND),
                    MODULES::XSS,
                ];
                let rate_c = rate.clone();
                for module in modules.iter() {
                    let rate = rate_c.clone(); 
                    let self_clone = Arc::clone(&self);
                    let task = tokio::spawn({
                        let module = module.clone();
                        async move {
                            self_clone.scan_module(&module, &rate).await;
                        }
                    });

                    tasks.push(task);
                }
            }
            OPTIONS::SELECTIVE(module, rate) => {
                let module = module.clone();  // Clone the module if needed (based on your enum types)
                let rate = rate.clone();
                let self_clone = Arc::clone(&self); 
                let task = tokio::spawn({
                    async move {
                        self_clone.scan_module(&module, &rate).await;
                    }
                });
                tasks.push(task);
            }
        }

        for task in tasks {
            task.await.unwrap();
        }
    }

    async fn scan_module(&self, module: &MODULES, rate: &RATE) {
        match module {
            MODULES::XSS => {
                XSSCAN { target: &self.customer }.run().await;
            }
            MODULES::SQLI(sql) => match sql {
                SQL::BLIND => {
                    sql_scanner { technique: SQL::BLIND, depth: rate }.run().await;
                }
                SQL::UNION => {
                    sql_scanner { technique: SQL::UNION, depth: rate }.run().await;
                }
                SQL::XML => {
                    sql_scanner { technique: SQL::XML, depth: rate}.run().await;
                }
            },
            _=> {}
        }
    }
}

#[derive(Debug,Clone)]

enum OPTIONS {
    FULLSCAN(RATE),
    SELECTIVE(MODULES, RATE),
}

#[derive(Debug,Clone)]
enum MODULES {
    XSS,
    SQLI(SQL),
    SMUGGLING,
    HOSTINGER,
    WORDPRESS,
    OSI,
    PHP,
}

#[derive(Debug,Clone)]
enum SQL {
    BLIND,
    UNION,
    XML,
}

#[derive(Debug,Clone)]
enum RATE {
    BLAZE,
    FAST,
    MODERATE,
    SLOW,
}

async fn jalap(url: String,endpoint: &str, cookie: Option<String>) {
    
    let res = jiber(&format!("{}/{}", url,endpoint)).await.unwrap();
    let crawldar = crawler::parse(&res.0);

    let semaphore = Arc::new(Semaphore::new(100)); 
    let mut tasks = vec![];
    let mut another_tasks = vec![];
    for data in crawldar.inner.into_iter() {
        let url_c = url.clone();
        let cookie_c = cookie.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let task = tokio::spawn(async move {
            let res = jiber(&format!("{}/{}", url_c,data)).await.unwrap();
            drop(permit);
            println!("{:?}",res);
            res
        });
        tasks.push(task);
    }

    let responses: Vec<_> = join_all(tasks).await; // Wait for all tasks concurrently
    println!("SHIT {:?}",responses);
    let mut overall_crawls = vec![];
    let mut crawl_bays = vec![];
    for resp in responses {
        let resp = resp.unwrap(); // Handle any potential errors here
        let copy_res = &resp.0.to_string();
        let methods = get_methods(resp.0);
        println!("{:?}",methods);
        overall_crawls.push(methods);
        let cr = crawler::parse(copy_res);
        crawl_bays.push(cr)
    }
     
    let combined = CrawlLink::combine(crawl_bays);
    for data in combined.inner.into_iter() {
        let url_c = url.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let cookie_c = cookie.clone();
        let task = tokio::spawn(async move {
            let res = jiber(&format!("{}/{}", url_c,data)).await.unwrap();
            drop(permit);
            println!("{:?}",res);
            res
        });
        another_tasks.push(task);
    }
    let responses: Vec<_> = join_all(another_tasks).await;
    for request in responses {
          let resp = request.unwrap().0;
          let method = get_methods(resp);
          println!(
            "Parsed method is {:?}",method
          );
          overall_crawls.push(method);;

    } 
    let suzgilenmic_methodlar: Vec<_> = overall_crawls.iter().flatten().collect();
    println!("{:?}",suzgilenmic_methodlar);

    //println!("ALL CRAWLED DATA {:?}", overall_crawls);
    //println!("CRAWL {:?}",crawl_bays);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 100)]
async fn main() {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        sender.send(5).unwrap();
    });
    
    let url = "https://sozdaniesajta.ru".to_string();
    println!("Received: {}", receiver.recv().unwrap());
    //get all links 
    //let res = smuggle(url.clone(),"/htmli_post.php","GET",Some("Cookie: security_level=0; PHPSESSID=vnk0b7acs3q4mku80jk1d3bq36".to_string()),Some("".to_string())).await.unwrap_or(("".to_string(),"".to_string()));
    //let crawldar = crawler::parse(&res.1);
    let all_shit = jalap(url.to_string(),"/",Some("Cookie: security_level=0; PHPSESSID=d7eplpene3o78evvqgtnbf21p0".to_string())).await;
    println!("QOTAQ {:?}",all_shit);
    //println!("{:?}",suraw::jiber(&url).await);
    // Example usage: perform a selective scan of SQLI Blind at BLAZE speed
    let scan = SCANER::new(
        1,
        OPTIONS::SELECTIVE(MODULES::SQLI(SQL::BLIND), RATE::BLAZE),  // Correct usage
        CUSTOMER::URL("https://middlecomm.kz".to_string()),
        RATE::BLAZE,
    );

    Arc::new(scan).start_scan();
}
