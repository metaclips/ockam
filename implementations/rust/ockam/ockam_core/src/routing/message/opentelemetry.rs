use crate::errcode::{Kind, Origin};
use core::fmt::{Display, Formatter};
use core::str::FromStr;
use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry::{global, Context};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Registry;

const TRACE_CONTEXT_PROPAGATION_SPAN: &str = "trace context propagation";

/// Name of the global Ockam tracer
pub const OCKAM_TRACER_NAME: &str = "ockam";

/// Serializable data type to hold the opentelemetry propagation context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenTelemetryContext(HashMap<String, String>);

impl Hash for OpenTelemetryContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state)
    }
}

impl PartialOrd for OpenTelemetryContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpenTelemetryContext {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl OpenTelemetryContext {
    /// Recover an OpenTelemetry context from the currently serialized data
    pub fn extract(&self) -> Context {
        global::get_text_map_propagator(|propagator| propagator.extract(self))
    }

    /// Serialize the current OpenTelemetry context as OpenTelemetryContext
    pub fn inject(context: &Context) -> Self {
        global::get_text_map_propagator(|propagator| {
            let mut propagation_context = OpenTelemetryContext::empty();
            propagator.inject_context(context, &mut propagation_context);
            propagation_context
        })
    }

    /// Update the OpenTelemetryContext with the latest span id
    pub fn update(mut self) -> OpenTelemetryContext {
        let _guard = self.extract().attach();
        let updated = OpenTelemetryContext::current();
        self.0 = updated.0;
        self
    }

    /// Return the current OpenTelemetryContext
    pub fn current() -> OpenTelemetryContext {
        // In order to get the current OpenTelemetry context that is connected to the
        // current span, as instrumented with the #[instrument] attribute, we need to:
        //
        //   1. Create a temporary span.
        //   2. Get its data, given its id, from the global registry.
        //   3. In the span extensions we can find the OpenTelemetry context that is used to attribute span ids.
        //      That context contains the span id of the latest span created with OpenTelemetry.
        //      That span is not the dummy span created below but the latest span created with #[instrument] in the
        //      current call stack.
        //      Note that opentelemetry::Context::current() would return a Context which only contains the latest context
        //      created with `tracer::in_span(...)` which is at the root of this trace. This is why we have to dig deep
        //      in order to retrieve the correct span id.
        //   4. Remove the OtelData extension so that our dummy "trace context propagation span" doesn't get emitted.
        let span = tracing::trace_span!(TRACE_CONTEXT_PROPAGATION_SPAN);
        let mut result = None;
        tracing::dispatcher::get_default(|dispatcher| {
            if let Some(registry) = dispatcher.downcast_ref::<Registry>() {
                if let Some(id) = span.id() {
                    if let Some(span) = registry.span(&id) {
                        let mut extensions = span.extensions_mut();
                        if let Some(OtelData {
                            builder: _,
                            parent_cx,
                        }) = extensions.remove::<OtelData>()
                        {
                            result = Some(OpenTelemetryContext::inject(&parent_cx))
                        }
                    }
                }
            };
        });
        // If, for some reason, we cannot retrieve the proper tracing context, we use the latest known
        // OpenTelemetry context
        result.unwrap_or_else(|| OpenTelemetryContext::inject(&opentelemetry::Context::current()))
    }

    /// Set this OpenTelemetry context as the new parent context
    fn set_as_parent_context(self) {
        let parent_cx = self.extract();
        let span = tracing::trace_span!(TRACE_CONTEXT_PROPAGATION_SPAN);
        tracing::dispatcher::get_default(|dispatcher| {
            if let Some(registry) = dispatcher.downcast_ref::<Registry>() {
                if let Some(id) = span.id() {
                    if let Some(span) = registry.span(&id) {
                        if let Some(parent) = span.parent() {
                            let mut extensions = parent.extensions_mut();
                            if let Some(otel_data) = extensions.get_mut::<OtelData>() {
                                otel_data.parent_cx = parent_cx.clone();
                            }
                        }
                        {
                            let mut extensions = span.extensions_mut();
                            extensions.remove::<OtelData>();
                        }
                    }
                }
            };
        })
    }

    /// Return the current opentelemetry::Context
    pub fn current_context() -> Context {
        OpenTelemetryContext::current().extract()
    }

    /// Parse a serialized tracing context, set it as the current parent context
    /// and return the current OpenTelemetry context
    /// This function is use to start new traces when receiving a serialized OpenTelemetryContext
    /// from remote nodes.
    pub fn from_remote_context(tracing_context: &str) -> OpenTelemetryContext {
        let result: Option<OpenTelemetryContext> = tracing_context.try_into().ok();
        if let Some(tc) = result {
            tc.set_as_parent_context()
        };
        OpenTelemetryContext::current()
    }

    fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Return the keys and values for testing
    pub fn as_map(&self) -> HashMap<String, String> {
        self.0.clone()
    }
}

impl Display for OpenTelemetryContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&serde_json::to_string(&self).map_err(|_| core::fmt::Error)?)
    }
}

impl Injector for OpenTelemetryContext {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }
}

impl Extractor for OpenTelemetryContext {
    fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_owned();
        self.0.get(&key).map(|v| v.as_ref())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_ref()).collect()
    }
}

/// Parse the OpenTelemetry context from a String
impl TryFrom<&str> for OpenTelemetryContext {
    type Error = crate::Error;

    fn try_from(value: &str) -> crate::Result<Self> {
        opentelemetry_context_parser(value)
    }
}

impl FromStr for OpenTelemetryContext {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Parse the OpenTelemetry context from a String
impl TryFrom<String> for OpenTelemetryContext {
    type Error = crate::Error;

    fn try_from(value: String) -> crate::Result<Self> {
        opentelemetry_context_parser(&value)
    }
}

/// Parse the OpenTelemetry context from a String
pub fn opentelemetry_context_parser(input: &str) -> crate::Result<OpenTelemetryContext> {
    serde_json::from_str(input).map_err(|e| {
        crate::Error::new(
            Origin::Api,
            Kind::Serialization,
            format!("Invalid OpenTelemetry context: {input}. Got error: {e:?}"),
        )
    })
}
