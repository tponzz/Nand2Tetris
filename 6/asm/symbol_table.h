#include <map>
#include <string>

class SymbolTable
{
  public:
    SymbolTable(/* args */);
    ~SymbolTable();
    void AddEntry(const std::string& symbol, const size_t address);
    bool Contains(const std::string& symbol);
    size_t GetAddress(const std::string& symbol);

  private:
    std::map<std::string, size_t> tbl_;
};