import re
import sys
import os

def remove_blocks(filepath, patterns):
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    original_len = len(lines)
    original_size = os.path.getsize(filepath)
    
    # Sort patterns by their occurrence in the file (reverse order to avoid index shifting)
    matches = []
    for pattern in patterns:
        for i, line in enumerate(lines):
            # Use a slightly more flexible regex for the start
            if re.search(pattern, line):
                matches.append((pattern, i))
                break
    
    matches.sort(key=lambda x: x[1], reverse=True)
    
    removed_count = 0
    
    for pattern, start_idx in matches:
        # Find the end of the block
        brace_count = 0
        end_idx = start_idx
        
        # Find the first brace on the start line or subsequent lines
        found_brace = False
        for i in range(start_idx, min(start_idx + 20, len(lines))):
            for char in lines[i]:
                if char == '{':
                    brace_count += 1
                    found_brace = True
                elif char == '}':
                    brace_count -= 1
            
            if found_brace and brace_count == 0:
                end_idx = i
                break
            elif i == start_idx + 19 and not found_brace:
                print(f"Warning: No closing brace found for pattern '{pattern}' at line {start_idx + 1}")
                break
        
        # Calculate start of removal (including up to 2 blank lines before)
        remove_start = start_idx
        blank_count = 0
        while remove_start > 0 and lines[remove_start - 1].strip() == '':
            remove_start -= 1
            blank_count += 1
            if blank_count >= 2:
                break
        
        # Remove the block
        lines[remove_start:end_idx + 1] = []
        
        print(f"Removed block for '{pattern}' (lines {remove_start + 1}-{end_idx + 1})")
        removed_count += 1

    with open(filepath, 'w') as f:
        f.writelines(lines)
        
    new_size = os.path.getsize(filepath)
    print(f"\nOriginal size: {original_size} bytes")
    print(f"New size: {new_size} bytes")
    print(f"Original lines: {original_len}")
    print(f"Remaining lines: {len(lines)}")
    print(f"Lines removed: {original_len - len(lines)}")
    print(f"Blocks removed: {removed_count}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python _decompose.py <file.rs>")
        sys.exit(1)
        
    target_file = sys.argv[1]
    
    patterns = [
        r'fn phase_planet\(&mut self\)',
        r'pub fn register_coastal_water_column\(&mut self',
        r'pub fn coastal_column_count\(&self\)',
        r'pub fn coastal_water_level\(&self, x: i64, z: i64\)',
        r'fn apply_tide_offset\(&mut self\)',
        r'pub fn diplomacy_events\(&self\)',
        r'pub fn push_diplomacy_event\(&mut self',
        r'pub fn run_macro_diplomacy_event\(&mut self\)',
        r'pub fn emit_relation_threshold_event\(',
        r'fn phase_diplomacy\(&mut self\)',
        r'pub fn apply_player_diplomacy_action\(',
        r'fn tick_faction_relation_drift\(&mut self\)',
        r'fn phase_economy\(&mut self\)',
        r'fn tick_settlement_trade_flows\(&mut self\)',
        r'fn apply_settlement_flow\(&mut self',
        r'fn tick_trade_routes\(&mut self\)',
        r'struct SettlementMarketSetup',
        r'pub fn resource_market_key\(resource: ResourceType',
        r'fn route_resource\(goods: &str\)',
        r'fn resource_amount\(resources: &Resources',
        r'fn adjust_resource\(resources: &mut Resources'
    ]
    
    remove_blocks(target_file, patterns)
