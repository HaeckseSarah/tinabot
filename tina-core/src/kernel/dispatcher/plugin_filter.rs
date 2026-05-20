use mlua::Value;
use std::sync::Arc;
use tina_plugin_api::{ApiValue, FilterRegistry};

pub struct PluginFilterMatcher;

impl PluginFilterMatcher {
    pub fn eval_rule(
        operator: &str,
        payload_val: Value,
        filter_val: Value,
        registry: &Arc<FilterRegistry>,
    ) -> bool {
        // Operator "foo.bar" -> ("foo", "bar")
        if let Some(dot_idx) = operator.find('.') {
            let (p_id, filter_name) = operator.split_at(dot_idx);
            let filter_name = &filter_name[1..];

            let api_field_val = Self::to_api_value(payload_val);
            let api_expected_val = Self::to_api_value(filter_val);

            let all_filters = registry.filters.lock().unwrap();
            if let Some(plugin_map) = all_filters.get(p_id) {
                if let Some(rust_filter_closure) = plugin_map.get(filter_name) {
                    return rust_filter_closure(api_field_val, api_expected_val);
                }
            }
        }
        false
    }

    fn to_api_value(lua_val: Value) -> ApiValue {
        match lua_val {
            Value::String(s) => s
                .to_str()
                .ok()
                .map(|str| ApiValue::String(str.to_string()))
                .unwrap_or(ApiValue::None),
            Value::Integer(i) => ApiValue::Integer(i),
            Value::Boolean(b) => ApiValue::Boolean(b),
            _ => ApiValue::None,
        }
    }
}
