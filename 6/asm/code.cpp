#include "code.h"
#include <algorithm>
#include <iostream>
#include <map>
#include <string>
namespace Asm {

using Table = std::map<std::string, std::string>;

static const Table jumps = {
    { "null", "000" }, { "JGT", "001" }, { "JEQ", "010" }, { "JGE", "011" },
    { "JLT", "100" },  { "JNE", "101" }, { "JLE", "110" }, { "JMP", "111" },
};

static const Table dests = {
    { "null", "000" }, { "M", "001" },  { "D", "010" },  { "DM", "011" },
    { "A", "100" },    { "AM", "101" }, { "AD", "110" }, { "ADM", "111" },
};

static const Table comps = {
    { "0", "101010" },    { "1", "1111111" },   { "-1", "0111010" },  { "D", "0001100" },
    { "A", "0110000" },   { "!A", "0110001" },  { "!D", "0001101" },  { "-D", "0001111" },
    { "-A", "0110011" },  { "D+1", "0011111" }, { "A+1", "0110111" }, { "D-1", "0001110" },
    { "A-1", "0110010" }, { "D+A", "0000010" }, { "D-A", "0010011" }, { "A-D", "0000111" },
    { "D&A", "0000000" }, { "D|A", "0010101" }, { "M", "1110000" },   { "!M", "1110001" },
    { "M+1", "1110111" }, { "M-1", "1110010" }, { "D+M", "1000010" }, { "D-M", "1010011" },
    { "M-D", "1000111" }, { "D&M", "1000000" }, { "D|M", "1010101" },
};

template<size_t N>
static std::array<char, N>
Lookup(const Table& tbl, const std::string& mnc)
{
    std::string code;
    if (tbl.contains(mnc)) {
        code = tbl.at(mnc);
    }

    std::array<char, N> ret{};
    if (!code.empty() && code.size() <= (N - 1)) {
        std::ranges::copy(code, ret.begin());
        // null-termination
        ret[code.size()] = '\0';
    }

    return ret;
}

std::array<char, 4>
Dest(const std::string& mnemonic)
{
    std::array<char, 4> d = Lookup<4>(dests, mnemonic);
    return d;
}

std::array<char, 8>
Comp(const std::string& mnemonic)
{
    std::array<char, 8> c = Lookup<8>(comps, mnemonic);
    return c;
}

std::array<char, 4>
Jump(const std::string& mnemonic)
{
    std::array<char, 4> j = Lookup<4>(jumps, mnemonic);
    return j;
}

} // namespace Asm