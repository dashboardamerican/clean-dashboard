/* @ts-self-types="./energy_simulator.d.ts" */

/**
 * Battery dispatch mode
 * @enum {0 | 1 | 2}
 */
export const BatteryMode = Object.freeze({
    /**
     * Water-fill algorithm prioritizes shaving highest peaks
     */
    Default: 0, "0": "Default",
    /**
     * Binary search finds optimal constant peak shaving line
     */
    PeakShaver: 1, "1": "PeakShaver",
    /**
     * Two-pass: peak shaving + opportunistic dispatch
     */
    Hybrid: 2, "2": "Hybrid",
});

/**
 * Cost parameters for LCOE calculation
 */
export class CostParams {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CostParamsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_costparams_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get battery_embodied_emissions() {
        const ret = wasm.__wbg_get_costparams_battery_embodied_emissions(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_capex() {
        const ret = wasm.__wbg_get_costparams_ccs_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_capture_rate() {
        const ret = wasm.__wbg_get_costparams_ccs_capture_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_energy_penalty() {
        const ret = wasm.__wbg_get_costparams_ccs_energy_penalty(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_fixed_om() {
        const ret = wasm.__wbg_get_costparams_ccs_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_percentage() {
        const ret = wasm.__wbg_get_costparams_ccs_percentage(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get ccs_var_om() {
        const ret = wasm.__wbg_get_costparams_ccs_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_capex() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_embodied_emissions() {
        const ret = wasm.__wbg_get_costparams_clean_firm_embodied_emissions(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_fixed_om() {
        const ret = wasm.__wbg_get_costparams_clean_firm_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_fuel() {
        const ret = wasm.__wbg_get_costparams_clean_firm_fuel(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_itc() {
        const ret = wasm.__wbg_get_costparams_clean_firm_itc(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_land_direct() {
        const ret = wasm.__wbg_get_costparams_clean_firm_land_direct(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_land_total() {
        const ret = wasm.__wbg_get_costparams_clean_firm_land_total(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_lifetime() {
        const ret = wasm.__wbg_get_costparams_clean_firm_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get clean_firm_var_om() {
        const ret = wasm.__wbg_get_costparams_clean_firm_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {DepreciationMethod}
     */
    get depreciation_method() {
        const ret = wasm.__wbg_get_costparams_depreciation_method(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get discount_rate() {
        const ret = wasm.__wbg_get_costparams_discount_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get electricity_price() {
        const ret = wasm.__wbg_get_costparams_electricity_price(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get excess_power_price() {
        const ret = wasm.__wbg_get_costparams_excess_power_price(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_capex() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_emissions_factor() {
        const ret = wasm.__wbg_get_costparams_gas_emissions_factor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_fixed_om() {
        const ret = wasm.__wbg_get_costparams_gas_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_heat_rate() {
        const ret = wasm.__wbg_get_costparams_gas_heat_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_land_direct() {
        const ret = wasm.__wbg_get_costparams_gas_land_direct(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_leakage_rate() {
        const ret = wasm.__wbg_get_costparams_gas_leakage_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_lifetime() {
        const ret = wasm.__wbg_get_costparams_gas_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get gas_price() {
        const ret = wasm.__wbg_get_costparams_gas_price(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_var_om() {
        const ret = wasm.__wbg_get_costparams_gas_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get inflation_rate() {
        const ret = wasm.__wbg_get_costparams_inflation_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get methane_gwp() {
        const ret = wasm.__wbg_get_costparams_methane_gwp(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get monetization_rate() {
        const ret = wasm.__wbg_get_costparams_monetization_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get monetize_excess_depreciation() {
        const ret = wasm.__wbg_get_costparams_monetize_excess_depreciation(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get project_lifetime() {
        const ret = wasm.__wbg_get_costparams_project_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get solar_capex() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get solar_embodied_emissions() {
        const ret = wasm.__wbg_get_costparams_solar_embodied_emissions(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get solar_fixed_om() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get solar_itc() {
        const ret = wasm.__wbg_get_costparams_solar_itc(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get solar_land_direct() {
        const ret = wasm.__wbg_get_costparams_solar_land_direct(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get solar_lifetime() {
        const ret = wasm.__wbg_get_costparams_solar_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get solar_var_om() {
        const ret = wasm.__wbg_get_costparams_solar_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get storage_capex() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get storage_fixed_om() {
        const ret = wasm.__wbg_get_costparams_storage_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get storage_itc() {
        const ret = wasm.__wbg_get_costparams_storage_itc(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get storage_lifetime() {
        const ret = wasm.__wbg_get_costparams_storage_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get storage_var_om() {
        const ret = wasm.__wbg_get_costparams_storage_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get tax_rate() {
        const ret = wasm.__wbg_get_costparams_tax_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_capex() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_embodied_emissions() {
        const ret = wasm.__wbg_get_costparams_wind_embodied_emissions(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_fixed_om() {
        const ret = wasm.__wbg_get_costparams_wind_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_itc() {
        const ret = wasm.__wbg_get_costparams_wind_itc(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_land_direct() {
        const ret = wasm.__wbg_get_costparams_wind_land_direct(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_land_total() {
        const ret = wasm.__wbg_get_costparams_wind_land_total(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_lifetime() {
        const ret = wasm.__wbg_get_costparams_wind_lifetime(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get wind_var_om() {
        const ret = wasm.__wbg_get_costparams_wind_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set battery_embodied_emissions(arg0) {
        wasm.__wbg_set_costparams_battery_embodied_emissions(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_capex(arg0) {
        wasm.__wbg_set_costparams_ccs_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_capture_rate(arg0) {
        wasm.__wbg_set_costparams_ccs_capture_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_energy_penalty(arg0) {
        wasm.__wbg_set_costparams_ccs_energy_penalty(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_fixed_om(arg0) {
        wasm.__wbg_set_costparams_ccs_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_percentage(arg0) {
        wasm.__wbg_set_costparams_ccs_percentage(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set ccs_var_om(arg0) {
        wasm.__wbg_set_costparams_ccs_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_capex(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_embodied_emissions(arg0) {
        wasm.__wbg_set_costparams_clean_firm_embodied_emissions(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_fixed_om(arg0) {
        wasm.__wbg_set_costparams_clean_firm_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_fuel(arg0) {
        wasm.__wbg_set_costparams_clean_firm_fuel(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_itc(arg0) {
        wasm.__wbg_set_costparams_clean_firm_itc(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_land_direct(arg0) {
        wasm.__wbg_set_costparams_clean_firm_land_direct(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_land_total(arg0) {
        wasm.__wbg_set_costparams_clean_firm_land_total(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_lifetime(arg0) {
        wasm.__wbg_set_costparams_clean_firm_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_var_om(arg0) {
        wasm.__wbg_set_costparams_clean_firm_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {DepreciationMethod} arg0
     */
    set depreciation_method(arg0) {
        wasm.__wbg_set_costparams_depreciation_method(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set discount_rate(arg0) {
        wasm.__wbg_set_costparams_discount_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set electricity_price(arg0) {
        wasm.__wbg_set_costparams_electricity_price(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set excess_power_price(arg0) {
        wasm.__wbg_set_costparams_excess_power_price(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_capex(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_emissions_factor(arg0) {
        wasm.__wbg_set_costparams_gas_emissions_factor(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_fixed_om(arg0) {
        wasm.__wbg_set_costparams_gas_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_heat_rate(arg0) {
        wasm.__wbg_set_costparams_gas_heat_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_land_direct(arg0) {
        wasm.__wbg_set_costparams_gas_land_direct(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_leakage_rate(arg0) {
        wasm.__wbg_set_costparams_gas_leakage_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_lifetime(arg0) {
        wasm.__wbg_set_costparams_gas_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_price(arg0) {
        wasm.__wbg_set_costparams_gas_price(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_var_om(arg0) {
        wasm.__wbg_set_costparams_gas_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set inflation_rate(arg0) {
        wasm.__wbg_set_costparams_inflation_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set methane_gwp(arg0) {
        wasm.__wbg_set_costparams_methane_gwp(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set monetization_rate(arg0) {
        wasm.__wbg_set_costparams_monetization_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set monetize_excess_depreciation(arg0) {
        wasm.__wbg_set_costparams_monetize_excess_depreciation(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set project_lifetime(arg0) {
        wasm.__wbg_set_costparams_project_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_capex(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_embodied_emissions(arg0) {
        wasm.__wbg_set_costparams_solar_embodied_emissions(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_fixed_om(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_itc(arg0) {
        wasm.__wbg_set_costparams_solar_itc(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_land_direct(arg0) {
        wasm.__wbg_set_costparams_solar_land_direct(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_lifetime(arg0) {
        wasm.__wbg_set_costparams_solar_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set solar_var_om(arg0) {
        wasm.__wbg_set_costparams_solar_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set storage_capex(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set storage_fixed_om(arg0) {
        wasm.__wbg_set_costparams_storage_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set storage_itc(arg0) {
        wasm.__wbg_set_costparams_storage_itc(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set storage_lifetime(arg0) {
        wasm.__wbg_set_costparams_storage_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set storage_var_om(arg0) {
        wasm.__wbg_set_costparams_storage_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set tax_rate(arg0) {
        wasm.__wbg_set_costparams_tax_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_capex(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_embodied_emissions(arg0) {
        wasm.__wbg_set_costparams_wind_embodied_emissions(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_fixed_om(arg0) {
        wasm.__wbg_set_costparams_wind_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_itc(arg0) {
        wasm.__wbg_set_costparams_wind_itc(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_land_direct(arg0) {
        wasm.__wbg_set_costparams_wind_land_direct(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_land_total(arg0) {
        wasm.__wbg_set_costparams_wind_land_total(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_lifetime(arg0) {
        wasm.__wbg_set_costparams_wind_lifetime(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_var_om(arg0) {
        wasm.__wbg_set_costparams_wind_var_om(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) CostParams.prototype[Symbol.dispose] = CostParams.prototype.free;

/**
 * MACRS depreciation method
 * @enum {0 | 1 | 2}
 */
export const DepreciationMethod = Object.freeze({
    Macrs5: 0, "0": "Macrs5",
    Macrs15: 1, "1": "Macrs15",
    StraightLine: 2, "2": "StraightLine",
});

/**
 * ELCC calculation method
 * @enum {0 | 1 | 2}
 */
export const ElccMethod = Object.freeze({
    /**
     * Average (First-In): Simulate with only that resource, measure gas reduction
     */
    Average: 0, "0": "Average",
    /**
     * Marginal (Last-In): Add 10 MW to full portfolio, measure gas reduction / 10
     */
    Marginal: 1, "1": "Marginal",
    /**
     * Delta: Allocate portfolio interactive effect proportionally
     */
    Delta: 2, "2": "Delta",
});

/**
 * Result of a land-use calculation.
 */
export class LandUseResult {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LandUseResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_landuseresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get clean_firm_direct_acres() {
        const ret = wasm.__wbg_get_landuseresult_clean_firm_direct_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get clean_firm_total_acres() {
        const ret = wasm.__wbg_get_landuseresult_clean_firm_total_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Direct (physical-footprint) land use in acres.
     * @returns {number}
     */
    get direct_acres() {
        const ret = wasm.__wbg_get_landuseresult_direct_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Direct land use in mi² (Python's headline number).
     * @returns {number}
     */
    get direct_mi2() {
        const ret = wasm.__wbg_get_landuseresult_direct_mi2(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_direct_acres() {
        const ret = wasm.__wbg_get_landuseresult_gas_direct_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get gas_total_acres() {
        const ret = wasm.__wbg_get_landuseresult_gas_total_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Per-technology direct contributions (acres). Useful for charts.
     * @returns {number}
     */
    get solar_direct_acres() {
        const ret = wasm.__wbg_get_landuseresult_solar_direct_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Per-technology total contributions (acres).
     * Solar and gas have no significant indirect footprint, so total == direct
     * for those two.
     * @returns {number}
     */
    get solar_total_acres() {
        const ret = wasm.__wbg_get_landuseresult_solar_total_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total (direct + indirect, e.g. wind spacing) land use in acres.
     * @returns {number}
     */
    get total_acres() {
        const ret = wasm.__wbg_get_landuseresult_total_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total land use in mi².
     * @returns {number}
     */
    get total_mi2() {
        const ret = wasm.__wbg_get_landuseresult_total_mi2(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_direct_acres() {
        const ret = wasm.__wbg_get_landuseresult_wind_direct_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get wind_total_acres() {
        const ret = wasm.__wbg_get_landuseresult_wind_total_acres(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_direct_acres(arg0) {
        wasm.__wbg_set_landuseresult_clean_firm_direct_acres(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set clean_firm_total_acres(arg0) {
        wasm.__wbg_set_landuseresult_clean_firm_total_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Direct (physical-footprint) land use in acres.
     * @param {number} arg0
     */
    set direct_acres(arg0) {
        wasm.__wbg_set_landuseresult_direct_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Direct land use in mi² (Python's headline number).
     * @param {number} arg0
     */
    set direct_mi2(arg0) {
        wasm.__wbg_set_landuseresult_direct_mi2(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_direct_acres(arg0) {
        wasm.__wbg_set_landuseresult_gas_direct_acres(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set gas_total_acres(arg0) {
        wasm.__wbg_set_landuseresult_gas_total_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Per-technology direct contributions (acres). Useful for charts.
     * @param {number} arg0
     */
    set solar_direct_acres(arg0) {
        wasm.__wbg_set_landuseresult_solar_direct_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Per-technology total contributions (acres).
     * Solar and gas have no significant indirect footprint, so total == direct
     * for those two.
     * @param {number} arg0
     */
    set solar_total_acres(arg0) {
        wasm.__wbg_set_landuseresult_solar_total_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Total (direct + indirect, e.g. wind spacing) land use in acres.
     * @param {number} arg0
     */
    set total_acres(arg0) {
        wasm.__wbg_set_landuseresult_total_acres(this.__wbg_ptr, arg0);
    }
    /**
     * Total land use in mi².
     * @param {number} arg0
     */
    set total_mi2(arg0) {
        wasm.__wbg_set_landuseresult_total_mi2(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_direct_acres(arg0) {
        wasm.__wbg_set_landuseresult_wind_direct_acres(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set wind_total_acres(arg0) {
        wasm.__wbg_set_landuseresult_wind_total_acres(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) LandUseResult.prototype[Symbol.dispose] = LandUseResult.prototype.free;

/**
 * LCOE calculation result with detailed breakdown
 */
export class LcoeResult {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LcoeResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_lcoeresult_free(ptr, 0);
    }
    /**
     * CCS cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get ccs_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_ccs_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * CCS LCOE contribution $/MWh
     * @returns {number}
     */
    get ccs_lcoe() {
        const ret = wasm.__wbg_get_costparams_wind_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Clean firm cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get clean_firm_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_clean_firm_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * Clean firm LCOE contribution $/MWh
     * @returns {number}
     */
    get clean_firm_lcoe() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Direct land use acres (physical footprint only)
     * @returns {number}
     */
    get direct_land_use() {
        const ret = wasm.__wbg_get_costparams_solar_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Emissions intensity g CO2/kWh
     * @returns {number}
     */
    get emissions_intensity() {
        const ret = wasm.__wbg_get_costparams_gas_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Gas cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get gas_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_gas_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * Gas LCOE contribution $/MWh
     * @returns {number}
     */
    get gas_lcoe() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total present value of costs $
     * @returns {number}
     */
    get pv_total_costs() {
        const ret = wasm.__wbg_get_costparams_storage_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total present value of energy MWh
     * @returns {number}
     */
    get pv_total_energy() {
        const ret = wasm.__wbg_get_costparams_clean_firm_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Solar cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get solar_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_solar_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * Solar LCOE contribution $/MWh
     * @returns {number}
     */
    get solar_lcoe() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Storage cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get storage_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_storage_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * Storage LCOE contribution $/MWh
     * @returns {number}
     */
    get storage_lcoe() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total land use acres (includes indirect: wind spacing, exclusion zones)
     * @returns {number}
     */
    get total_land_use() {
        const ret = wasm.__wbg_get_costparams_wind_var_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total system LCOE in $/MWh
     * @returns {number}
     */
    get total_lcoe() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Wind cost breakdown
     * @returns {TechnologyCostBreakdown}
     */
    get wind_breakdown() {
        const ret = wasm.__wbg_get_lcoeresult_wind_breakdown(this.__wbg_ptr);
        return TechnologyCostBreakdown.__wrap(ret);
    }
    /**
     * Wind LCOE contribution $/MWh
     * @returns {number}
     */
    get wind_lcoe() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * CCS cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set ccs_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_ccs_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * CCS LCOE contribution $/MWh
     * @param {number} arg0
     */
    set ccs_lcoe(arg0) {
        wasm.__wbg_set_costparams_wind_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Clean firm cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set clean_firm_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_clean_firm_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * Clean firm LCOE contribution $/MWh
     * @param {number} arg0
     */
    set clean_firm_lcoe(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Direct land use acres (physical footprint only)
     * @param {number} arg0
     */
    set direct_land_use(arg0) {
        wasm.__wbg_set_costparams_solar_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * Emissions intensity g CO2/kWh
     * @param {number} arg0
     */
    set emissions_intensity(arg0) {
        wasm.__wbg_set_costparams_gas_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Gas cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set gas_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_gas_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * Gas LCOE contribution $/MWh
     * @param {number} arg0
     */
    set gas_lcoe(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Total present value of costs $
     * @param {number} arg0
     */
    set pv_total_costs(arg0) {
        wasm.__wbg_set_costparams_storage_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Total present value of energy MWh
     * @param {number} arg0
     */
    set pv_total_energy(arg0) {
        wasm.__wbg_set_costparams_clean_firm_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Solar cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set solar_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_solar_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * Solar LCOE contribution $/MWh
     * @param {number} arg0
     */
    set solar_lcoe(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Storage cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set storage_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_storage_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * Storage LCOE contribution $/MWh
     * @param {number} arg0
     */
    set storage_lcoe(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Total land use acres (includes indirect: wind spacing, exclusion zones)
     * @param {number} arg0
     */
    set total_land_use(arg0) {
        wasm.__wbg_set_costparams_wind_var_om(this.__wbg_ptr, arg0);
    }
    /**
     * Total system LCOE in $/MWh
     * @param {number} arg0
     */
    set total_lcoe(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Wind cost breakdown
     * @param {TechnologyCostBreakdown} arg0
     */
    set wind_breakdown(arg0) {
        _assertClass(arg0, TechnologyCostBreakdown);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_lcoeresult_wind_breakdown(this.__wbg_ptr, ptr0);
    }
    /**
     * Wind LCOE contribution $/MWh
     * @param {number} arg0
     */
    set wind_lcoe(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) LcoeResult.prototype[Symbol.dispose] = LcoeResult.prototype.free;

/**
 * Optimizer configuration
 */
export class OptimizerConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OptimizerConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_optimizerconfig_free(ptr, 0);
    }
    /**
     * Battery round-trip efficiency used during optimizer evaluations
     * @returns {number}
     */
    get battery_efficiency() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Enable clean firm in optimization
     * @returns {boolean}
     */
    get enable_clean_firm() {
        const ret = wasm.__wbg_get_optimizerconfig_enable_clean_firm(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable solar in optimization
     * @returns {boolean}
     */
    get enable_solar() {
        const ret = wasm.__wbg_get_optimizerconfig_enable_solar(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable storage in optimization
     * @returns {boolean}
     */
    get enable_storage() {
        const ret = wasm.__wbg_get_optimizerconfig_enable_storage(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable wind in optimization
     * @returns {boolean}
     */
    get enable_wind() {
        const ret = wasm.__wbg_get_optimizerconfig_enable_wind(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Maximum clean firm capacity MW
     * @returns {number}
     */
    get max_clean_firm() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum demand response used during optimizer evaluations
     * @returns {number}
     */
    get max_demand_response() {
        const ret = wasm.__wbg_get_costparams_wind_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum solar capacity MW
     * @returns {number}
     */
    get max_solar() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum storage capacity MWh
     * @returns {number}
     */
    get max_storage() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum wind capacity MW
     * @returns {number}
     */
    get max_wind() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Target clean match percentage (0-100)
     * @returns {number}
     */
    get target_clean_match() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Battery round-trip efficiency used during optimizer evaluations
     * @param {number} arg0
     */
    set battery_efficiency(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Enable clean firm in optimization
     * @param {boolean} arg0
     */
    set enable_clean_firm(arg0) {
        wasm.__wbg_set_optimizerconfig_enable_clean_firm(this.__wbg_ptr, arg0);
    }
    /**
     * Enable solar in optimization
     * @param {boolean} arg0
     */
    set enable_solar(arg0) {
        wasm.__wbg_set_optimizerconfig_enable_solar(this.__wbg_ptr, arg0);
    }
    /**
     * Enable storage in optimization
     * @param {boolean} arg0
     */
    set enable_storage(arg0) {
        wasm.__wbg_set_optimizerconfig_enable_storage(this.__wbg_ptr, arg0);
    }
    /**
     * Enable wind in optimization
     * @param {boolean} arg0
     */
    set enable_wind(arg0) {
        wasm.__wbg_set_optimizerconfig_enable_wind(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum clean firm capacity MW
     * @param {number} arg0
     */
    set max_clean_firm(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum demand response used during optimizer evaluations
     * @param {number} arg0
     */
    set max_demand_response(arg0) {
        wasm.__wbg_set_costparams_wind_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum solar capacity MW
     * @param {number} arg0
     */
    set max_solar(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum storage capacity MWh
     * @param {number} arg0
     */
    set max_storage(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum wind capacity MW
     * @param {number} arg0
     */
    set max_wind(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Target clean match percentage (0-100)
     * @param {number} arg0
     */
    set target_clean_match(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) OptimizerConfig.prototype[Symbol.dispose] = OptimizerConfig.prototype.free;

/**
 * Optimizer result
 */
export class OptimizerResult {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OptimizerResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_optimizerresult_free(ptr, 0);
    }
    /**
     * Achieved clean match percentage
     * @returns {number}
     */
    get achieved_clean_match() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Optimal clean firm capacity MW
     * @returns {number}
     */
    get clean_firm_capacity() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Resulting LCOE $/MWh
     * @returns {number}
     */
    get lcoe() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Number of evaluations
     * @returns {number}
     */
    get num_evaluations() {
        const ret = wasm.__wbg_get_optimizerresult_num_evaluations(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Optimal solar capacity MW
     * @returns {number}
     */
    get solar_capacity() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Optimal storage capacity MWh
     * @returns {number}
     */
    get storage_capacity() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Optimization successful
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_optimizerresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Optimal wind capacity MW
     * @returns {number}
     */
    get wind_capacity() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Achieved clean match percentage
     * @param {number} arg0
     */
    set achieved_clean_match(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Optimal clean firm capacity MW
     * @param {number} arg0
     */
    set clean_firm_capacity(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Resulting LCOE $/MWh
     * @param {number} arg0
     */
    set lcoe(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Number of evaluations
     * @param {number} arg0
     */
    set num_evaluations(arg0) {
        wasm.__wbg_set_optimizerresult_num_evaluations(this.__wbg_ptr, arg0);
    }
    /**
     * Optimal solar capacity MW
     * @param {number} arg0
     */
    set solar_capacity(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Optimal storage capacity MWh
     * @param {number} arg0
     */
    set storage_capacity(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Optimization successful
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_optimizerresult_success(this.__wbg_ptr, arg0);
    }
    /**
     * Optimal wind capacity MW
     * @param {number} arg0
     */
    set wind_capacity(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) OptimizerResult.prototype[Symbol.dispose] = OptimizerResult.prototype.free;

/**
 * ORDC configuration parameters
 */
export class OrdcConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OrdcConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ordcconfig_free(ptr, 0);
    }
    /**
     * Steepness parameter (lambda)
     * @returns {number}
     */
    get lambda() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum price cap $/MWh
     * @returns {number}
     */
    get max_price() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Reserve requirement as % of load
     * @returns {number}
     */
    get reserve_requirement() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Steepness parameter (lambda)
     * @param {number} arg0
     */
    set lambda(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum price cap $/MWh
     * @param {number} arg0
     */
    set max_price(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Reserve requirement as % of load
     * @param {number} arg0
     */
    set reserve_requirement(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) OrdcConfig.prototype[Symbol.dispose] = OrdcConfig.prototype.free;

/**
 * Electricity pricing method
 * @enum {0 | 1 | 2 | 3}
 */
export const PricingMethod = Object.freeze({
    /**
     * SRMC + capacity adder during tight supply, scaled to match LCOE
     */
    ScarcityBased: 0, "0": "ScarcityBased",
    /**
     * Pure energy-only market (SRMC)
     */
    MarginalCost: 1, "1": "MarginalCost",
    /**
     * Operating Reserve Demand Curve pricing
     */
    Ordc: 2, "2": "Ordc",
    /**
     * Dual revenue stream: energy + capacity payments
     */
    MarginalPlusCapacity: 3, "3": "MarginalPlusCapacity",
});

/**
 * Configuration for a simulation run
 */
export class SimulationConfig {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SimulationConfig.prototype);
        obj.__wbg_ptr = ptr;
        SimulationConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SimulationConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_simulationconfig_free(ptr, 0);
    }
    /**
     * Battery round-trip efficiency (0-1)
     * @returns {number}
     */
    get battery_efficiency() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Battery dispatch mode
     * @returns {BatteryMode}
     */
    get battery_mode() {
        const ret = wasm.__wbg_get_simulationconfig_battery_mode(this.__wbg_ptr);
        return ret;
    }
    /**
     * Clean firm capacity in MW (constant output)
     * @returns {number}
     */
    get clean_firm_capacity() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Maximum demand response in MW
     * @returns {number}
     */
    get max_demand_response() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Solar capacity in MW
     * @returns {number}
     */
    get solar_capacity() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Storage capacity in MWh
     * @returns {number}
     */
    get storage_capacity() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Wind capacity in MW
     * @returns {number}
     */
    get wind_capacity() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Battery round-trip efficiency (0-1)
     * @param {number} arg0
     */
    set battery_efficiency(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Battery dispatch mode
     * @param {BatteryMode} arg0
     */
    set battery_mode(arg0) {
        wasm.__wbg_set_simulationconfig_battery_mode(this.__wbg_ptr, arg0);
    }
    /**
     * Clean firm capacity in MW (constant output)
     * @param {number} arg0
     */
    set clean_firm_capacity(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Maximum demand response in MW
     * @param {number} arg0
     */
    set max_demand_response(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Solar capacity in MW
     * @param {number} arg0
     */
    set solar_capacity(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Storage capacity in MWh
     * @param {number} arg0
     */
    set storage_capacity(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Wind capacity in MW
     * @param {number} arg0
     */
    set wind_capacity(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} solar_capacity
     * @param {number} wind_capacity
     * @param {number} storage_capacity
     * @param {number} clean_firm_capacity
     * @param {number} battery_efficiency
     * @param {number} max_demand_response
     * @param {BatteryMode} battery_mode
     */
    constructor(solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, battery_efficiency, max_demand_response, battery_mode) {
        const ret = wasm.simulationconfig_new(solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, battery_efficiency, max_demand_response, battery_mode);
        this.__wbg_ptr = ret >>> 0;
        SimulationConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {SimulationConfig}
     */
    static with_defaults() {
        const ret = wasm.simulationconfig_with_defaults();
        return SimulationConfig.__wrap(ret);
    }
}
if (Symbol.dispose) SimulationConfig.prototype[Symbol.dispose] = SimulationConfig.prototype.free;

/**
 * Cost breakdown for a single technology
 */
export class TechnologyCostBreakdown {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TechnologyCostBreakdown.prototype);
        obj.__wbg_ptr = ptr;
        TechnologyCostBreakdownFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TechnologyCostBreakdownFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_technologycostbreakdown_free(ptr, 0);
    }
    /**
     * Capital expenditure $/MWh
     * @returns {number}
     */
    get capex() {
        const ret = wasm.__wbg_get_costparams_solar_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Fixed O&M $/MWh
     * @returns {number}
     */
    get fixed_om() {
        const ret = wasm.__wbg_get_costparams_wind_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Fuel cost $/MWh
     * @returns {number}
     */
    get fuel() {
        const ret = wasm.__wbg_get_costparams_clean_firm_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * ITC benefit (negative = reduces cost) $/MWh
     * @returns {number}
     */
    get itc_benefit() {
        const ret = wasm.__wbg_get_costparams_gas_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Depreciation tax shield (negative = reduces cost) $/MWh
     * @returns {number}
     */
    get tax_shield() {
        const ret = wasm.__wbg_get_costparams_solar_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total for this technology $/MWh
     * @returns {number}
     */
    get total() {
        const ret = wasm.__wbg_get_costparams_wind_fixed_om(this.__wbg_ptr);
        return ret;
    }
    /**
     * Variable O&M $/MWh
     * @returns {number}
     */
    get var_om() {
        const ret = wasm.__wbg_get_costparams_storage_capex(this.__wbg_ptr);
        return ret;
    }
    /**
     * Capital expenditure $/MWh
     * @param {number} arg0
     */
    set capex(arg0) {
        wasm.__wbg_set_costparams_solar_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Fixed O&M $/MWh
     * @param {number} arg0
     */
    set fixed_om(arg0) {
        wasm.__wbg_set_costparams_wind_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Fuel cost $/MWh
     * @param {number} arg0
     */
    set fuel(arg0) {
        wasm.__wbg_set_costparams_clean_firm_capex(this.__wbg_ptr, arg0);
    }
    /**
     * ITC benefit (negative = reduces cost) $/MWh
     * @param {number} arg0
     */
    set itc_benefit(arg0) {
        wasm.__wbg_set_costparams_gas_capex(this.__wbg_ptr, arg0);
    }
    /**
     * Depreciation tax shield (negative = reduces cost) $/MWh
     * @param {number} arg0
     */
    set tax_shield(arg0) {
        wasm.__wbg_set_costparams_solar_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Total for this technology $/MWh
     * @param {number} arg0
     */
    set total(arg0) {
        wasm.__wbg_set_costparams_wind_fixed_om(this.__wbg_ptr, arg0);
    }
    /**
     * Variable O&M $/MWh
     * @param {number} arg0
     */
    set var_om(arg0) {
        wasm.__wbg_set_costparams_storage_capex(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) TechnologyCostBreakdown.prototype[Symbol.dispose] = TechnologyCostBreakdown.prototype.free;

/**
 * @returns {BatteryMode}
 */
export function battery_mode_default() {
    const ret = wasm.battery_mode_default();
    return ret;
}

/**
 * @returns {BatteryMode}
 */
export function battery_mode_hybrid() {
    const ret = wasm.battery_mode_hybrid();
    return ret;
}

/**
 * @returns {BatteryMode}
 */
export function battery_mode_peak_shaver() {
    const ret = wasm.battery_mode_peak_shaver();
    return ret;
}

/**
 * Calculate ELCC metrics for all resources
 *
 * # Arguments
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 * * `solar_profile` - Solar capacity factors (Float64Array)
 * * `wind_profile` - Wind capacity factors (Float64Array)
 * * `load_profile` - Load MW (Float64Array)
 * * `battery_mode_js` - Battery mode as string
 * * `battery_efficiency` - Battery round-trip efficiency
 * * `max_demand_response` - Maximum demand response fraction
 *
 * # Returns
 * * ElccResult as JsValue
 * @param {number} solar_capacity
 * @param {number} wind_capacity
 * @param {number} storage_capacity
 * @param {number} clean_firm_capacity
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} battery_mode_js
 * @param {number} battery_efficiency
 * @param {number} max_demand_response
 * @returns {any}
 */
export function calculate_elcc_metrics(solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, solar_profile, wind_profile, load_profile, battery_mode_js, battery_efficiency, max_demand_response) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.calculate_elcc_metrics(solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, ptr0, len0, ptr1, len1, ptr2, len2, battery_mode_js, battery_efficiency, max_demand_response);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Calculate land use for a portfolio without running the simulation.
 *
 * # Arguments
 * * `solar_capacity` - Solar capacity (MW)
 * * `wind_capacity` - Wind capacity (MW)
 * * `clean_firm_capacity` - Clean firm capacity (MW)
 * * `gas_capacity` - Peak gas capacity needed (MW). Pass the same value
 *   you would read from `SimulationResult.peak_gas`.
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * LandUseResult as JsValue with `direct_acres`, `total_acres`,
 *   `direct_mi2`, `total_mi2`, plus per-technology breakdowns.
 * @param {number} solar_capacity
 * @param {number} wind_capacity
 * @param {number} clean_firm_capacity
 * @param {number} gas_capacity
 * @param {any} costs_js
 * @returns {any}
 */
export function compute_land_use(solar_capacity, wind_capacity, clean_firm_capacity, gas_capacity, costs_js) {
    const ret = wasm.compute_land_use(solar_capacity, wind_capacity, clean_firm_capacity, gas_capacity, costs_js);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Calculate LCOE for a simulation result
 *
 * # Arguments
 * * `sim_result_js` - SimulationResult as JsValue
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * LcoeResult as JsValue
 * @param {any} sim_result_js
 * @param {number} solar_capacity
 * @param {number} wind_capacity
 * @param {number} storage_capacity
 * @param {number} clean_firm_capacity
 * @param {any} costs_js
 * @returns {any}
 */
export function compute_lcoe(sim_result_js, solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, costs_js) {
    const ret = wasm.compute_lcoe(sim_result_js, solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity, costs_js);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Compute hourly electricity prices
 *
 * # Arguments
 * * `sim_result_js` - SimulationResult as JsValue
 * * `costs_js` - CostParams as JsValue
 * * `lcoe` - System LCOE $/MWh
 * * `pricing_method_js` - PricingMethod as JsValue
 * * `load_profile` - Load MW (Float64Array)
 * * `ordc_config_js` - Optional OrdcConfig as JsValue
 * * `elcc_result_js` - Optional ElccResult as JsValue
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 *
 * # Returns
 * * PricingResult as JsValue
 * @param {any} sim_result_js
 * @param {any} costs_js
 * @param {number} lcoe
 * @param {any} pricing_method_js
 * @param {Float64Array} load_profile
 * @param {any} ordc_config_js
 * @param {any} elcc_result_js
 * @param {number} solar_capacity
 * @param {number} wind_capacity
 * @param {number} storage_capacity
 * @param {number} clean_firm_capacity
 * @returns {any}
 */
export function compute_prices(sim_result_js, costs_js, lcoe, pricing_method_js, load_profile, ordc_config_js, elcc_result_js, solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity) {
    const ptr0 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.compute_prices(sim_result_js, costs_js, lcoe, pricing_method_js, ptr0, len0, ordc_config_js, elcc_result_js, solar_capacity, wind_capacity, storage_capacity, clean_firm_capacity);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Evaluate a batch of portfolios (for Web Worker parallel processing)
 *
 * # Arguments
 * * `portfolios_js` - Array of portfolio configurations
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `battery_mode` - Battery dispatch mode
 * * `config_js` - Optional OptimizerConfig as JsValue for runtime assumptions
 *
 * # Returns
 * * Array of evaluation results
 * @param {any} portfolios_js
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {BatteryMode} battery_mode
 * @param {any | null} [config_js]
 * @returns {any}
 */
export function evaluate_batch(portfolios_js, solar_profile, wind_profile, load_profile, costs_js, battery_mode, config_js) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.evaluate_batch(portfolios_js, ptr0, len0, ptr1, len1, ptr2, len2, costs_js, battery_mode, isLikeNone(config_js) ? 0 : addToExternrefTable0(config_js));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get default cost parameters
 * @returns {any}
 */
export function get_default_costs() {
    const ret = wasm.get_default_costs();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get default optimizer config
 * @returns {any}
 */
export function get_default_optimizer_config() {
    const ret = wasm.get_default_optimizer_config();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get default simulation config
 * @returns {any}
 */
export function get_default_simulation_config() {
    const ret = wasm.get_default_simulation_config();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get the library version
 * @returns {string}
 */
export function get_version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.get_version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Initialize panic hook for better error messages in browser console
 */
export function init() {
    wasm.init();
}

/**
 * Run the optimizer (V2 hierarchical optimizer)
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 *
 * Note: If a model is loaded for the current zone/mode via `wasm_load_model()`,
 * it will be used automatically for faster candidate filtering.
 * @param {number} target_match
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function optimize(target_match, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.optimize(target_match, ptr0, len0, ptr1, len1, ptr2, len2, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run optimizer sweep with model-based acceleration
 *
 * Uses cached model for faster candidate filtering if available.
 * Returns the same SweepResult structure as run_optimizer_sweep.
 *
 * # Arguments
 * * `zone` - Zone name (must match loaded model)
 * * `targets` - Array of target percentages
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * SweepResult as JsValue (same format as run_optimizer_sweep)
 * @param {string} zone
 * @param {Float64Array} targets
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function optimize_sweep_with_model(zone, targets, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passStringToWasm0(zone, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(targets, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ptr4 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len4 = WASM_VECTOR_LEN;
    const ret = wasm.optimize_sweep_with_model(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run the V2 hierarchical optimizer
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 * @param {number} target_match
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function optimize_v2(target_match, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.optimize_v2(target_match, ptr0, len0, ptr1, len1, ptr2, len2, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run V2 optimizer sweep across multiple targets
 *
 * # Arguments
 * * `targets` - Array of target percentages
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * Array of OptimizerResult as JsValue
 * @param {Float64Array} targets
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function optimize_v2_sweep(targets, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passArrayF64ToWasm0(targets, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.optimize_v2_sweep(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run optimizer with model-based acceleration (if model is cached)
 *
 * This is the preferred method when you have loaded a model via `wasm_load_model()`.
 * Falls back to greedy search if no model is cached for the zone/mode.
 *
 * # Arguments
 * * `zone` - Zone name (must match the zone used when loading the model)
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 * @param {string} zone
 * @param {number} target_match
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function optimize_with_model(zone, target_match, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passStringToWasm0(zone, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.optimize_with_model(ptr0, len0, target_match, ptr1, len1, ptr2, len2, ptr3, len3, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run cost sweep - optimize across a range of parameter values
 *
 * # Arguments
 * * `target_match` - Target clean match percentage
 * * `param_name` - Name of parameter to sweep
 * * `min_value` - Minimum parameter value
 * * `max_value` - Maximum parameter value
 * * `steps` - Number of steps in sweep
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `base_costs_js` - Base CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * CostSweepResult as JsValue
 * @param {number} target_match
 * @param {string} param_name
 * @param {number} min_value
 * @param {number} max_value
 * @param {number} steps
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} base_costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function run_cost_sweep(target_match, param_name, min_value, max_value, steps, solar_profile, wind_profile, load_profile, base_costs_js, config_js, battery_mode) {
    const ptr0 = passStringToWasm0(param_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.run_cost_sweep(target_match, ptr0, len0, min_value, max_value, steps, ptr1, len1, ptr2, len2, ptr3, len3, base_costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run cost sweep with model-based acceleration
 *
 * Uses cached model for faster candidate filtering if available.
 *
 * # Arguments
 * * `zone` - Zone name (must match loaded model)
 * * `target_match` - Target clean match percentage
 * * `param_name` - Name of parameter to sweep
 * * `min_value` - Minimum parameter value
 * * `max_value` - Maximum parameter value
 * * `steps` - Number of steps in sweep
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `base_costs_js` - Base CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * CostSweepResult as JsValue
 * @param {string} zone
 * @param {number} target_match
 * @param {string} param_name
 * @param {number} min_value
 * @param {number} max_value
 * @param {number} steps
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} base_costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function run_cost_sweep_with_model(zone, target_match, param_name, min_value, max_value, steps, solar_profile, wind_profile, load_profile, base_costs_js, config_js, battery_mode) {
    const ptr0 = passStringToWasm0(zone, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(param_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ptr4 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len4 = WASM_VECTOR_LEN;
    const ret = wasm.run_cost_sweep_with_model(ptr0, len0, target_match, ptr1, len1, min_value, max_value, steps, ptr2, len2, ptr3, len3, ptr4, len4, base_costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run the incremental cost walk optimizer.
 *
 * Mirrors the Python `run_incremental_cost_walk` strategy: starts from a zero
 * portfolio and incrementally adds the most cost-effective resource (smallest
 * LCOE-per-percentage-point ratio) until reaching the clean-match target,
 * halving step sizes when overshooting.
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (values >= 100 are capped to 99.5)
 * * `solar_profile` - Solar capacity factors (8760 hours)
 * * `wind_profile` - Wind capacity factors (8760 hours)
 * * `load_profile` - Load MW (8760 hours)
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue (provides battery_efficiency,
 *   max_demand_response, and the resource-enable flags)
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * IncrementalWalkResult as JsValue (includes the full walk_trace)
 * @param {number} target_match
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function run_incremental_walk_wasm(target_match, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.run_incremental_walk_wasm(target_match, ptr0, len0, ptr1, len1, ptr2, len2, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run optimizer sweep and return structured result (uses V2 optimizer)
 * @param {Float64Array} targets
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @param {any} config_js
 * @param {BatteryMode} battery_mode
 * @returns {any}
 */
export function run_optimizer_sweep(targets, solar_profile, wind_profile, load_profile, costs_js, config_js, battery_mode) {
    const ptr0 = passArrayF64ToWasm0(targets, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.run_optimizer_sweep(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, costs_js, config_js, battery_mode);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run a single simulation and return results as JSON
 *
 * # Arguments
 * * `config_js` - SimulationConfig as JsValue
 * * `solar_profile` - Solar capacity factors (Float64Array)
 * * `wind_profile` - Wind capacity factors (Float64Array)
 * * `load_profile` - Load MW (Float64Array)
 *
 * # Returns
 * * SimulationResult as JsValue (JSON-serializable)
 * @param {any} config_js
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @returns {any}
 */
export function simulate(config_js, solar_profile, wind_profile, load_profile) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.simulate(config_js, ptr0, len0, ptr1, len1, ptr2, len2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Run full simulation and LCOE calculation in one call
 *
 * # Arguments
 * * `config_js` - SimulationConfig as JsValue
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * Object with both simulation and LCOE results
 * @param {any} config_js
 * @param {Float64Array} solar_profile
 * @param {Float64Array} wind_profile
 * @param {Float64Array} load_profile
 * @param {any} costs_js
 * @returns {any}
 */
export function simulate_and_calculate_lcoe(config_js, solar_profile, wind_profile, load_profile, costs_js) {
    const ptr0 = passArrayF64ToWasm0(solar_profile, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(wind_profile, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayF64ToWasm0(load_profile, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.simulate_and_calculate_lcoe(config_js, ptr0, len0, ptr1, len1, ptr2, len2, costs_js);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get model cache statistics
 *
 * # Returns
 * * Object with { loaded: number, max: number }
 * @returns {any}
 */
export function wasm_cache_stats() {
    const ret = wasm.wasm_cache_stats();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Clear all cached models to free memory
 *
 * Call this when switching contexts or to reduce memory usage.
 * Models will need to be reloaded before model-based optimization can be used.
 */
export function wasm_clear_models() {
    wasm.wasm_clear_models();
}

/**
 * Check if a model is loaded in the cache
 *
 * # Arguments
 * * `zone` - Zone name (case-insensitive)
 * * `battery_mode` - Battery mode
 *
 * # Returns
 * * `true` if model is cached and ready for use
 * * `false` if model needs to be loaded
 * @param {string} zone
 * @param {BatteryMode} battery_mode
 * @returns {boolean}
 */
export function wasm_is_model_loaded(zone, battery_mode) {
    const ptr0 = passStringToWasm0(zone, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.wasm_is_model_loaded(ptr0, len0, battery_mode);
    return ret !== 0;
}

/**
 * Load an empirical model into the cache for model-based optimization
 *
 * # Arguments
 * * `zone` - Zone name (case-insensitive, e.g., "california", "texas")
 * * `battery_mode` - Battery mode (must match the mode used to generate the model)
 * * `bytes` - Model binary data (bincode serialized EmpiricalModel)
 *
 * # Returns
 * * `Ok(())` if model loaded successfully
 * * `Err` if deserialization fails
 *
 * # Example (TypeScript)
 * ```typescript
 * const response = await fetch('/models/california_hybrid.bin');
 * const bytes = new Uint8Array(await response.arrayBuffer());
 * wasm.wasm_load_model('california', BatteryMode.Hybrid, bytes);
 * ```
 * @param {string} zone
 * @param {BatteryMode} battery_mode
 * @param {Uint8Array} bytes
 */
export function wasm_load_model(zone, battery_mode, bytes) {
    const ptr0 = passStringToWasm0(zone, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.wasm_load_model(ptr0, len0, battery_mode, ptr1, len1);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * Get list of currently loaded models
 *
 * # Returns
 * * Array of [zone, battery_mode] pairs as JSON
 * @returns {any}
 */
export function wasm_loaded_models() {
    const ret = wasm.wasm_loaded_models();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_Number_04624de7d0e8332d: function(arg0) {
            const ret = Number(arg0);
            return ret;
        },
        __wbg_String_8f0eb39a4a4c2f66: function(arg0, arg1) {
            const ret = String(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_47fa6863be6f2f25: function(arg0, arg1) {
            const ret = arg0 in arg1;
            return ret;
        },
        __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_ac34f5003991759a: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_object_5ae8e5880f2c1fbd: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_cd444516edc5b180: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_9dd77d8cd6671811: function(arg0, arg1) {
            const ret = arg0 == arg1;
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_done_57b39ecd9addfe81: function(arg0) {
            const ret = arg0.done;
            return ret;
        },
        __wbg_entries_58c7934c745daac7: function(arg0) {
            const ret = Object.entries(arg0);
            return ret;
        },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_get_9b94d73e6221f75c: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_with_ref_key_1dc361bd10053bfe: function(arg0, arg1) {
            const ret = arg0[arg1];
            return ret;
        },
        __wbg_instanceof_ArrayBuffer_c367199e2fa2aa04: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_9b9075935c74707c: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_d314bb98fcf08331: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_isSafeInteger_bfbc7332a9768d2a: function(arg0) {
            const ret = Number.isSafeInteger(arg0);
            return ret;
        },
        __wbg_iterator_6ff6560ca1568e55: function() {
            const ret = Symbol.iterator;
            return ret;
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_35a7bace40f36eac: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_next_3482f54c49e8af19: function() { return handleError(function (arg0) {
            const ret = arg0.next();
            return ret;
        }, arguments); },
        __wbg_next_418f80d8f5303233: function(arg0) {
            const ret = arg0.next;
            return ret;
        },
        __wbg_now_a3af9a2f4bbaa4d1: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_prototypesetcall_bdcdcc5842e4d77d: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_set_3f1d0b984ed272ed: function(arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        },
        __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_value_0546255b415e96c1: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
        __wbindgen_object_is_undefined: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
    };
    return {
        __proto__: null,
        "./energy_simulator_bg.js": import0,
    };
}

const CostParamsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_costparams_free(ptr >>> 0, 1));
const LandUseResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_landuseresult_free(ptr >>> 0, 1));
const LcoeResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_lcoeresult_free(ptr >>> 0, 1));
const OptimizerConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_optimizerconfig_free(ptr >>> 0, 1));
const OptimizerResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_optimizerresult_free(ptr >>> 0, 1));
const OrdcConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ordcconfig_free(ptr >>> 0, 1));
const SimulationConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_simulationconfig_free(ptr >>> 0, 1));
const TechnologyCostBreakdownFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_technologycostbreakdown_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('energy_simulator_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
