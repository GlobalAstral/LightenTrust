#include <iostream>
#include <sstream>
#include <fstream>
#include <vector>
#include <algorithm>
#include <sstream>

#include <Utils/Constants.hpp>
#include <Tokenizer/Tokenizer.hpp>
#include <Parser/Parser.hpp>

using std::cout;
using std::endl;
using std::string;
using std::vector;
using std::stringstream;
using std::ifstream;
using std::ofstream;

int main(int argc, char** argv) {
  vector<string> args{};
  for (int i = 1; i < argc; i++) {
    args.push_back(string(argv[i]));
  }

  auto asm_index = std::find(args.begin(), args.end(), "-asm");
  auto obj_index = std::find(args.begin(), args.end(), "-obj");
  bool keep_asm = asm_index != args.end();
  bool keep_obj = obj_index != args.end();
  if (keep_asm)
    args.erase(asm_index);
  if (keep_obj)
    args.erase(obj_index);

  if (args.size() <= 0)
    return 1;
  string i_file = args.at(0);
  if (i_file.substr(i_file.size()-3) != EXTENSION)
    return 1;
  string obj_file = string(i_file).erase(i_file.size()-2) + "obj";
  string exe_file = (args.size() > 1) ? args.at(1) : string(i_file).erase(i_file.size()-2) + "exe";

  stringstream content;
  ifstream ifile{i_file};
  string buf;
  while (std::getline(ifile, buf)) {
    content << buf << "\n";
  }
  ifile.close();

  Tokenizer::Tokenizer tokenizer(content.str());
  vector<Tokens::Token> tokens = tokenizer.tokenize();
  
  cout << endl << "TOKENS:" << endl;

  for (Tokens::Token token : tokens) {
    cout << token.toString() << endl;
  }

  Parser::Parser parser{tokens};
  vector<Node::NodeInstance*> nodes = parser.parse();

  cout << endl << "NODES:" << endl;

  for (Node::NodeInstance* instance : nodes) {
    cout << instance->toString() << endl;
  }

	return 0;
}
