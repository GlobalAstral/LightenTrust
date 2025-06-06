#pragma once

#include <iostream>
#include <vector>

namespace Map {
  template <typename K, typename V>
  class Map {
    public:
      bool contains(K key) {
        return find(key) > -1;
      }
      V& operator[](K key) {
        int index = find(key);
        if (index < 0) {
          keys.push_back(key);
          values.push_back(*(new V));
          return values.back();
        }
        return values[index];
      }
      int remove(K key) {
        int index = find(key);
        if (index < 0)
          return 1;
        this->keys.erase(this->keys.begin()+index);
        this->values.erase(this->values.begin()+index);
        return 0;
      }
      int size() {
        return this->keys.size();
      }
      K getKey(int index) {
        if (index < 0 || index >= size())
          return {};
        return this->keys[index];
      }

      std::vector<K> getKeys() {
        return this->keys;
      }
    private:
      int find(K key) {
        for (int i = 0; i < keys.size(); i++) {
          if (keys.at(i) == key)
            return i;
        }
        return -1;
      }
      std::vector<K> keys{};
      std::vector<V> values{};
  };
}
