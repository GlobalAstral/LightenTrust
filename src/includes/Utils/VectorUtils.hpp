#pragma once

#include <iostream>
#include <vector>

namespace VectorUtils {
  template <typename T>
  int find(std::vector<T> vec, T item) {
    return find<T>(vec, item, [](T a, T b){
      return a == b;
    });
  }

  template <typename T>
  int find(std::vector<T> vec, T item, std::function<bool(T,T)> criteria) {
    int half = vec.size() / 2;
    int spare = vec.size() % 2 != 0;
  
    if (spare > 0 && criteria(vec.at(vec.size()-1), item))
      return vec.size()-1;
    
    for (int left = 0; left < half; left++) {
      int right = vec.size()-left-spare-1;
      if (criteria(vec[left], item))
        return left;
      if (criteria(vec[right], item))
        return right;
    }
  
    return -1;
  }

}

