#include "code.h"
#include <algorithm>
#include <iostream>
#include <map>
#include <string>
namespace Asm {

using Table = std::map<std::string, std::string>;

static const Table jumps = {
    { "JGT", "001" }, { "JEQ", "010" }, { "JGE", "011" }, { "JLT", "100" },
    { "JNE", "101" }, { "JLE", "110" }, { "JMP", "111" },
};

static const Table dests = {
    { "M", "001" },  { "D", "010" },  { "DM", "011" },  { "A", "100" },
    { "AM", "101" }, { "AD", "110" }, { "ADM", "111" },
};

static const Table comps = { { "0", "101010" },    { "1", "111111" },   { "-1", "111010" },
                             { "D", "001100" },   { "A", "110000" },   { "!A", "110001" },
                             { "!D", "001101" },  { "-D", "001111" },  { "-A", "110011" },
                             { "D+1", "011111" }, { "A+1", "110111" }, { "D-1", "001110" },
                             { "A-1", "110010" }, { "D+A", "000010" }, { "D-A", "010011" },
                             { "A-D", "000111" }, { "D&A", "000000" }, { "D|A", "010101" },
                             { "M", "110000" },   { "!M", "110001" },  { "M+1", "110111" },
                             { "M-1", "110010" }, { "D+M", "000010" }, { "D-M", "010011" },
                             { "M-D", "000111" }, { "D&M", "000000" }, { "D|M", "010101" } };

std::string
Dest(const std::string& mnemonic)
{
    if (dests.find(mnemonic) != dests.end()) {
        return dests.at(mnemonic);
    }
    // default: no destination -> "000"
    return std::string("000");
}

std::string
Comp(const std::string& mnemonic)
{
    if (comps.find(mnemonic) != comps.end()) {
        return comps.at(mnemonic);
    }
    // default: no destination -> "000000"
    return std::string("000000");
}

std::string
Jump(const std::string& mnemonic)
{
    if (jumps.find(mnemonic) != jumps.end()) {
        return jumps.at(mnemonic);
    }
    // default: no destination -> "000000"
    return std::string("000");
}

} // namespace Asm