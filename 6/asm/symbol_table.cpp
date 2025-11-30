#include "symbol_table.h"

#include <iostream>
#include <limits>

SymbolTable::SymbolTable()
{
    // Predefined symbols from the Nand2Tetris spec
    tbl_.emplace("SP", 0);
    tbl_.emplace("LCL", 1);
    tbl_.emplace("ARG", 2);
    tbl_.emplace("THIS", 3);
    tbl_.emplace("THAT", 4);

    // R0-R15 registers
    for (size_t i = 0; i <= 15; ++i) {
        tbl_.emplace(std::string("R") + std::to_string(i), i);
    }

    // Memory-mapped I/O
    tbl_.emplace("SCREEN", 16384);
    tbl_.emplace("KBD", 24576);
}
SymbolTable::~SymbolTable() {}

void
SymbolTable::AddEntry(const std::string& symbol, const size_t address)
{
    // Only add when symbol is not already present.
    if (!Contains(symbol)) {
        const auto&& [it, ok] = tbl_.emplace(symbol, address);
        if (!ok) {
            std::cerr << "[WARN] Symbol not inserted: " << symbol << std::endl;
        }
    }
}

bool
SymbolTable::Contains(const std::string& symbol)
{
    return tbl_.contains(symbol);
}

size_t
SymbolTable::GetAddress(const std::string& symbol)
{
    if (Contains(symbol))
    {
        return tbl_.at(symbol);
    }
    
    return std::numeric_limits<size_t>::max();
}
