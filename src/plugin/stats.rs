// Copyright 2024 Tree xie.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{get_hash_key, get_step_conf, get_str_conf, Error, Plugin, Result};
use crate::config::{PluginCategory, PluginConf, PluginStep};
use crate::http_extra::HttpResponse;
use crate::state::{
    get_hostname, get_process_system_info, get_processing_accepted,
    get_start_time, State,
};
use crate::util;
use async_trait::async_trait;
use bytes::Bytes;
use pingora::proxy::Session;
use serde::Serialize;
use std::time::Duration;
use tracing::debug;

#[derive(Serialize)]
struct ServerStats {
    processing: i32,
    accepted: u64,
    location_processing: i32,
    location_accepted: u64,
    hostname: String,
    version: String,
    rustc_version: String,
    start_time: u64,
    uptime: String,
    memory_mb: usize,
    memory: String,
    arch: String,
    cpus: usize,
    physical_cpus: usize,
    total_memory: String,
    used_memory: String,
    threads: usize,
    fd_count: usize,
    tcp_count: usize,
    tcp6_count: usize,
}
pub struct Stats {
    path: String,
    plugin_step: PluginStep,
    hash_value: String,
}

impl TryFrom<&PluginConf> for Stats {
    type Error = Error;
    fn try_from(value: &PluginConf) -> Result<Self> {
        let hash_value = get_hash_key(value);
        let step = get_step_conf(value);

        let params = Self {
            hash_value,
            plugin_step: step,
            path: get_str_conf(value, "path"),
        };
        if ![PluginStep::Request, PluginStep::ProxyUpstream]
            .contains(&params.plugin_step)
        {
            return Err(Error::Invalid {
                category: PluginCategory::Stats.to_string(),
                message: "Stats plugin should be executed at request or proxy upstream step".to_string(),
            });
        }
        Ok(params)
    }
}

impl Stats {
    pub fn new(params: &PluginConf) -> Result<Self> {
        debug!(params = params.to_string(), "new stats plugin");
        Self::try_from(params)
    }
}

#[async_trait]
impl Plugin for Stats {
    #[inline]
    fn hash_key(&self) -> String {
        self.hash_value.clone()
    }
    #[inline]
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut State,
    ) -> pingora::Result<Option<HttpResponse>> {
        if step != self.plugin_step {
            return Ok(None);
        }
        if session.req_header().uri.path() == self.path {
            let uptime: humantime::Duration =
                Duration::from_secs(util::now().as_secs() - get_start_time())
                    .into();
            let (processing, accepted) = get_processing_accepted();
            let info = get_process_system_info();
            let resp = HttpResponse::try_from_json(&ServerStats {
                accepted,
                processing,
                location_processing: ctx.location_processing,
                location_accepted: ctx.location_accepted,
                hostname: get_hostname().to_string(),
                version: util::get_pkg_version().to_string(),
                rustc_version: util::get_rustc_version(),
                start_time: get_start_time(),
                uptime: uptime.to_string(),
                memory_mb: info.memory_mb,
                memory: info.memory,
                arch: info.arch,
                cpus: info.cpus,
                physical_cpus: info.physical_cpus,
                total_memory: info.total_memory,
                used_memory: info.used_memory,
                threads: info.threads,
                fd_count: info.fd_count,
                tcp_count: info.tcp_count,
                tcp6_count: info.tcp6_count,
            })
            .unwrap_or_else(|e| {
                HttpResponse::unknown_error(Bytes::from(e.to_string()))
            });
            return Ok(Some(resp));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;
    use crate::state::State;
    use crate::{config::PluginConf, config::PluginStep, plugin::Plugin};
    use pingora::proxy::Session;
    use pretty_assertions::assert_eq;
    use tokio_test::io::Builder;

    #[test]
    fn test_stats_params() {
        let params = Stats::try_from(
            &toml::from_str::<PluginConf>(
                r###"
        path = "/stats"
    "###,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!("/stats", params.path);

        let result = Stats::try_from(
            &toml::from_str::<PluginConf>(
                r###"
        step = "response"
        path = "/stats"
    "###,
            )
            .unwrap(),
        );

        assert_eq!(
            "Plugin stats invalid, message: Stats plugin should be executed at request or proxy upstream step",
            result.err().unwrap().to_string()
        );
    }

    #[tokio::test]
    async fn test_stats() {
        let stats = Stats::new(
            &toml::from_str::<PluginConf>(
                r###"
            path = "/stats"
        "###,
            )
            .unwrap(),
        )
        .unwrap();

        let headers = ["Accept-Encoding: gzip"].join("\r\n");
        let input_header =
            format!("GET /vicanso/pingap?size=1 HTTP/1.1\r\n{headers}\r\n\r\n");
        let mock_io = Builder::new().read(input_header.as_bytes()).build();
        let mut session = Session::new_h1(Box::new(mock_io));
        session.read_request().await.unwrap();

        let result = stats
            .handle_request(
                PluginStep::Request,
                &mut session,
                &mut State::default(),
            )
            .await
            .unwrap();
        assert_eq!(true, result.is_none());

        let headers = ["Accept-Encoding: gzip"].join("\r\n");
        let input_header = format!("GET /stats HTTP/1.1\r\n{headers}\r\n\r\n");
        let mock_io = Builder::new().read(input_header.as_bytes()).build();
        let mut session = Session::new_h1(Box::new(mock_io));
        session.read_request().await.unwrap();

        let result = stats
            .handle_request(
                PluginStep::Request,
                &mut session,
                &mut State::default(),
            )
            .await
            .unwrap();
        assert_eq!(true, result.is_some());
    }
}
