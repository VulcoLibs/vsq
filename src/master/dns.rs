use std::net::IpAddr;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::{IntoName, TokioAsyncResolver, TryParseIp};

pub struct DNS {
    resolver: TokioAsyncResolver,
}

impl DNS {
    pub async fn new() -> DNS {
        let resolver = tokio::runtime::Handle::current().spawn(async {
            TokioAsyncResolver::tokio(
                ResolverConfig::default(),
                ResolverOpts::default())
        }).await.unwrap();

        DNS {
            resolver
        }
    }

    pub async fn lookup_ip<N: IntoName + TryParseIp>(&self, host: N) -> Option<IpAddr> {
        let response = self.resolver.lookup_ip(host).await.ok()?;
        Some(response.iter().next()?)
    }
}
