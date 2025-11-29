#include <array>
#include <string>

namespace Asm {

std::array<char, 4>
Dest(const std::string& mnemonic);

std::array<char, 8>
Comp(const std::string& mnemonic);

std::array<char, 4>
Jump(const std::string& mnemonic);

} // namespace Asm
