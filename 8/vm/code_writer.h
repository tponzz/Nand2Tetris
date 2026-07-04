#ifndef VM_CODE_WRITER_HH

#include <fstream>
#include <string>

#include "parser.h"

namespace Vm {

class CodeWriter
{
  public:
    CodeWriter(const std::string& filepath);
    ~CodeWriter();

    void SetFileName(const std::string& filename);

    void WriteArithmetic(const std::string& cmd_line);
    void WritePushPop(Parser::Cmd cmd, const std::string& seg, const size_t idx);
    void WriteLabel(const std::string& label);
    void WriteGoto(const std::string& label);
    void WriteIf(const std::string& label);
    void WriteFuntion(const std::string& function_name, const int n_vars);
    void WriteCall(const std::string& label, const int n_vars);
    void WriteReturn();

    void Close();

  private:
    std::ofstream _out;
    std::string _filename;
};

} // namespace Vm

#define VM_CODE_WRITER_HH
#endif
