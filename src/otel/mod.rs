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

use async_trait::async_trait;
use opentelemetry::{
    global::{self, BoxedTracer},
    propagation::TextMapCompositePropagator,
    trace::TracerProvider,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::{self, BatchConfig, RandomIdGenerator, Sampler},
    Resource,
};
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use std::time::Duration;
use tracing::{error, info};

pub struct TracerService {
    name: String,
    endpoint: String,
}

impl TracerService {
    pub fn new(name: &str, endpoint: &str) -> TracerService {
        Self {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
        }
    }
}

#[inline]
fn get_service_name(name: &str) -> String {
    format!("pingap-{name}")
}

#[inline]
pub fn new_tracer(name: &str) -> Option<BoxedTracer> {
    if let Some(provider) = provider::get_provider(name) {
        return Some(provider.tracer(get_service_name(name)));
    }
    None
}

#[async_trait]
impl BackgroundService for TracerService {
    /// The lets encrypt servier checks the cert, it will get news cert if current is invalid.
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let result = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&self.endpoint)
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(
                trace::Config::default()
                    // TODO smapler config
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_attributes_per_span(16)
                    .with_max_events_per_span(16)
                    .with_resource(Resource::new(vec![KeyValue::new(
                        "service.name",
                        get_service_name(&self.name),
                    )])),
            )
            .with_batch_config(BatchConfig::default())
            .install_batch(opentelemetry_sdk::runtime::Tokio);

        match result {
            Ok(tracer_provider) => {
                info!(endpoint = self.endpoint, "opentelemetry init success");
                let baggage_propagator = BaggagePropagator::new();
                let trace_context_propagator = TraceContextPropagator::new();
                let jaeger_propagator =
                    opentelemetry_jaeger_propagator::Propagator::new();
                global::set_text_map_propagator(
                    TextMapCompositePropagator::new(vec![
                        Box::new(trace_context_propagator),
                        Box::new(baggage_propagator),
                        Box::new(jaeger_propagator),
                    ]),
                );

                // set tracer provider
                provider::add_provider(&self.name, tracer_provider.clone());

                let _ = shutdown.changed().await;
                if let Err(e) = tracer_provider.shutdown() {
                    error!(
                        error = e.to_string(),
                        "opentelemetry shutdown fail"
                    );
                } else {
                    info!("opentelemetry shutdown success");
                }
            },
            Err(e) => {
                error!(error = e.to_string(), "opentelemetry init fail");
            },
        }
    }
}

mod provider;