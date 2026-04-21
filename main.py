import numpy as np
import pandas as pd

# =====================================================================
# БЛОК ВВОДА ДАННЫХ ДЛЯ ВАРИАНТА 3
# =====================================================================

def f(x):
    """Целевая функция f(x)"""
    return (x[0] - 6)**2 + (x[1] + 8)**2

def g(x):
    """Ограничение g(x) = x1^2 - x2 <= 0"""
    return x[0]**2 - x[1]

def barrier_function(x, mu):
    """
    Вспомогательная барьерная функция (согласно Варианту 3)
    P(x, mu) = f(x) - mu / g(x)
    """
    g_val = g(x)
    # Если вышли за границу или на границу, возвращаем бесконечность (барьер)
    if g_val >= -1e-9: 
        return np.inf
    return f(x) - mu / g_val

# =====================================================================
# УЛУЧШЕННЫЙ МЕТОД ЦИКЛИЧЕСКОГО ПОКООРДИНАТНОГО СПУСКА
# =====================================================================

def coordinate_descent(x_start, mu, eps_inner):
    """
    Поиск минимума P(x, mu) методом циклического покоординатного спуска.
    Используется адаптивный шаг для поиска точного минимума по каждой оси.
    """
    x = np.array(x_start, dtype=float)
    n = len(x)
    history = []
    
    # Параметры внутреннего поиска
    max_cycles = 100 
    
    for cycle in range(max_cycles):
        x_old_cycle = x.copy()
        
        for i in range(n):
            # Внутренний цикл: минимизация по одной координате x[i]
            # Используем процедуру скользящего окна с уменьшением шага
            h = 1.0 # Начальный шаг поиска
            for _ in range(20): # Глубина уточнения координаты
                improved = True
                while improved:
                    improved = False
                    current_val = barrier_function(x, mu)
                    
                    # Пробуем шаг вправо и влево
                    for direction in [1, -1]:
                        x_new = x.copy()
                        x_new[i] += direction * h
                        new_val = barrier_function(x_new, mu)
                        
                        if new_val < current_val:
                            x[i] = x_new[i]
                            current_val = new_val
                            improved = True
                h /= 2 # Уменьшаем шаг для точности
                
            # Запись шага в таблицу
            history.append({
                "Итерация": cycle + 1,
                "Координата": f"x{i+1}",
                "Точка X": list(np.round(x, 4)),
                "P(X, mu)": np.round(barrier_function(x, mu), 4)
            })
            
        # Условие выхода из покоординатного спуска (цикл по всем x завершен)
        if np.linalg.norm(x - x_old_cycle) < eps_inner:
            break
            
    return x, pd.DataFrame(history)

# =====================================================================
# ОСНОВНОЙ ЦИКЛ МЕТОДА БАРЬЕРНЫХ ФУНКЦИЙ (СТРАТЕГИЯ C)
# =====================================================================

def solve_barrier_method(x_init, mu_init, beta, epsilon):
    x_k = np.array(x_init, dtype=float)
    mu = mu_init
    k = 1
    
    print(f"ПАРАМЕТРЫ: mu_start={mu_init}, beta={beta}, eps={epsilon}, X0={x_init}\n")
    
    while True:
        print(f">>> ВНЕШНЯЯ ИТЕРАЦИЯ {k} (mu = {mu})")
        
        # Шаг 1: Находим минимум P(x, mu)
        x_next, table = coordinate_descent(x_k, mu, 0.00001)
        print(table.to_string(index=False))
        
        # Шаг 2: Проверка критерия остановки: mu * B(x) < eps
        # B(x) = -1/g(x) для метода обратных функций
        val_kBx = -mu / g(x_next)
        
        print(f"\nПроверка: mu*B(x) = {val_kBx:.4f} (критерий < {epsilon})")
        
        if val_kBx < epsilon:
            print(f"Критерий выполнен на итерации {k}!")
            x_k = x_next
            break
        
        # Шаг 3: Обновление параметров
        x_k = x_next
        mu *= beta
        k += 1
        if k > 20: break # Защита от бесконечного цикла

    print("\n" + "="*60)
    print("ИТОГОВОЕ РЕШЕНИЕ:")
    print(f"X* = {x_k}")
    print(f"f(X*) = {f(x_k):.6f}")
    print(f"g(X*) = {g(x_k):.6f}")

# =====================================================================
# ЗАПУСК
# =====================================================================

if __name__ == "__main__":
    # Данные из вашего условия для проверки:
    solve_barrier_method(
        x_init=[0.0, 12.0], 
        mu_init=10.0, 
        beta=0.1, 
        epsilon=0.5
    )