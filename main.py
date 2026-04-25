import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from docx import Document
from docx.shared import Pt
from docx.enum.text import WD_ALIGN_PARAGRAPH

# Стратегии
a = 1
b = 2
c = 3
d = 4

# =====================================================================
# БЛОК ВВОДА ДАННЫХ ДЛЯ ВАРИАНТА 3
# =====================================================================

def f(x):
    """Целевая функция f(x)"""
    return (x[0] - 6)**2 + (x[1] + 8)**2

def g(x):
    """Ограничение g(x) = x1^2 - x2 <= 0"""
    return x[0]**2 - x[1]

def barrier_term(x):
    """B(x) = -1/g(x)"""
    val = g(x)
    return -1.0 / val if val < 0 else 1e10

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
# МЕТОД ЦИКЛИЧЕСКОГО ПОКООРДИНАТНОГО СПУСКА
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
                "Точка X": f"[{x[0]:.4f}, {x[1]:.4f}]",
                "P(X, mu)": f"{barrier_function(x, mu):.4f}"
            })
            
        # Условие выхода из покоординатного спуска (цикл по всем x завершен)
        if np.linalg.norm(x - x_old_cycle) < eps_inner:
            break

    df_history = pd.DataFrame(history)
    # Вывод промежуточной таблицы в терминал
    print(df_history.to_string(index=False))
            
    return x, df_history

# =====================================================================
# ВИЗУАЛИЗАЦИЯ
# =====================================================================

def plot_optimization_process(trajectory, final_point):
    trajectory = np.array(trajectory)
    
    # Определяем границы графика
    x_min, x_max = -5, 3
    y_min, y_max = -15, 10
    
    x_range = np.linspace(x_min, x_max, 400)
    y_range = np.linspace(y_min, y_max, 400)
    X, Y = np.meshgrid(x_range, y_range)
    Z = (X - 6)**2 + (Y + 8)**2
    
    plt.figure(figsize=(10, 8))
    
    # 1. Линии уровня функции
    levels = np.logspace(1, 3, 20)
    cp = plt.contour(X, Y, Z, levels=levels, colors='blue', alpha=0.3)
    plt.clabel(cp, inline=True, fontsize=8)
    
    # 2. Ограничение g(x) <= 0  => x1^2 - x2 <= 0 => x2 >= x1^2
    G = X**2 - Y
    plt.contour(X, Y, G, levels=[0], colors='red', linewidths=2)
    plt.fill_between(x_range, x_range**2, y_max, color='red', alpha=0.1, label='Допустимая область g(x)<=0')

    # 3. Траектория движения
    plt.plot(trajectory[:, 0], trajectory[:, 1], 'go-', markersize=5, label='Траектория (внешние итерации)')
    
    # Стрелки направления
    for i in range(len(trajectory)-1):
        plt.annotate('', xy=trajectory[i+1], xytext=trajectory[i],
                     arrowprops=dict(arrowstyle="->", color="black", lw=1))

    # Точки
    plt.scatter(trajectory[0,0], trajectory[0,1], c='black', s=100, label='Старт $X_0$', zorder=5)
    plt.scatter(final_point[0], final_point[1], c='gold', marker='*', s=200, label='Оптимум $X^*$', edgecolors='black', zorder=6)

    plt.title('Визуализация метода барьерных функций (Вариант 3)')
    plt.xlabel('$x_1$')
    plt.ylabel('$x_2$')
    plt.legend()
    plt.grid(True, linestyle='--', alpha=0.6)
    plt.xlim(x_min, x_max)
    plt.ylim(y_min, y_max)
    
    plt.savefig('optimization_plot.png')
    plt.show()

# =====================================================================
# ФОРМИРОВАНИЕ ОТЧЕТА DOCX
# =====================================================================

def generate_docx(all_data, summary_df, final_results, params):
    doc = Document()
    title = doc.add_heading('Отчет по методу барьерных функций', 0)
    title.alignment = WD_ALIGN_PARAGRAPH.CENTER

    # 1. Параметры
    doc.add_heading('1. Исходные параметры', level=1)
    p = doc.add_paragraph()
    p.add_run(f"Начальная точка X0: {params['x_init']}\n")
    p.add_run(f"Начальный параметр mu: {params['mu_init']}\n")
    p.add_run(f"Коэффициент уменьшения beta: {params['beta']}\n")
    p.add_run(f"Точность epsilon: {params['epsilon']}")

    # 2. Ход решения (Детальные таблицы)
    doc.add_heading('2. Детальный ход решения', level=1)
    for iteration_data in all_data:
        doc.add_heading(f"Внешняя итерация {iteration_data['k']} (mu = {iteration_data['mu']})", level=2)
        df = iteration_data['table']
        table = doc.add_table(rows=1, cols=len(df.columns))
        table.style = 'Table Grid'
        for i, column_name in enumerate(df.columns):
            table.rows[0].cells[i].text = column_name
        for _, row in df.iterrows():
            row_cells = table.add_row().cells
            for i, value in enumerate(row):
                row_cells[i].text = str(value)

    # 3. ФИНАЛЬНАЯ СВОДНАЯ ТАБЛИЦА (как на картинке)
    doc.add_heading('3. Сводная таблица результатов', level=1)
    final_table = doc.add_table(rows=1, cols=len(summary_df.columns))
    final_table.style = 'Table Grid'
    for i, col in enumerate(summary_df.columns):
        final_table.rows[0].cells[i].text = col
    for _, row in summary_df.iterrows():
        row_cells = final_table.add_row().cells
        for i, val in enumerate(row):
            row_cells[i].text = str(val)

    # 4. Итог
    doc.add_heading('4. Итоговый результат', level=1)
    res_p = doc.add_paragraph()
    res_p.add_run(f"Оптимальная точка X*: {final_results['x_star']}\n")
    res_p.add_run(f"f(X*): {final_results['f_val']:.6f}\n")
    res_p.add_run(f"g(X*): {final_results['g_val']:.6f}")

    doc.save("Optimization_Report.docx")
    print("\nОтчет 'Optimization_Report.docx' успешно сформирован.")

# =====================================================================
# ОСНОВНОЙ ЦИКЛ
# =====================================================================

def solve_barrier_method(x_init, mu_init, beta, epsilon, strategy):
    print(f"\n{'='*50}")
    print(f"СТРАТЕГИЯ: {strategy}")
    print(f"{'='*50}")

    # --- КРИТИЧЕСКАЯ ПРОВЕРКА НАХОЖДЕНИЯ ВНУТРИ ОБЛАСТИ ---
    g_start = g(x_init)
    if g_start >= 0:
        print(f"ОШИБКА: Начальная точка {x_init} находится ВНЕ допустимой области!")
        print(f"g(x) = {g_start:.4f} (должно быть < 0)")
        return None, None
    

    x_k = np.array(x_init, dtype=float)
    mu = mu_init
    k = 1
    
    trajectory = [x_k.copy()]
    all_iterations_for_report = []
    summary_data = [] # Для финальной таблицы
    params = {'x_init': x_init, 'mu_init': mu_init, 'beta': beta, 'epsilon': epsilon}

    print(f"--- ЗАПУСК АЛГОРИТМА ---")
    print(f"Параметры: mu={mu_init}, beta={beta}, eps={epsilon}, X0={x_init}\n")

    while True:
        print(f"\n>>> ВНЕШНЯЯ ИТЕРАЦИЯ {k} (mu = {mu:.4f})")
        
        # Шаг 1: Находим минимум P(x, mu)
        x_next, table = coordinate_descent(x_k, mu, 0.0001)
        
        f_val = f(x_next)
        mu_Bx = mu * barrier_term(x_next)
        
        # Сохраняем в сводную таблицу
        summary_data.append({
            "k": k,
            "mu_k": mu,
            "X_k": f"({x_k[0]:.5f}; {x_k[1]:.5f})",
            "f(X_k)": f"{f_val:.5f}",
            "B": f"{barrier_term(x_next):.5f}",
            "Teta": f"{(f_val + mu_Bx):.5f}",
            "mu*B(X_k)": f"{mu_Bx:.5f}"
        })

        # Шаг 2: Проверка критерия остановки
        val_kBx = -mu / g(x_next)
        print(f"\nПроверка критерия: mu*B(x) = {val_kBx:.5f} (нужно < {epsilon})")
        
        # Сохраняем данные для отчета
        all_iterations_for_report.append({'k': k, 'mu': mu, 'table': table, 'check': mu_Bx})
        trajectory.append(x_next.copy())

        # Вывод в терминал
        print(f"\n--- Итерация {k} ---")
        print(f"X_{k} = {x_k}, f(X_{k}) = {f_val:.5f}, B = {barrier_term(x_next):.5f}, Teta = {(f_val + mu_Bx):.5f}, mu*B = {mu_Bx:.5f}")

        if mu_Bx < epsilon:
            x_k = x_next
            break
        
        x_k = x_next

        if strategy == a:
            mu = 0.01
            #mu *= 0.001
        elif strategy == b:
            mu = 0.01
        elif strategy == c:
            mu *= 0.1
        elif strategy == d:
            mu = 0.001
        else:
            mu *= beta
        k += 1
        #if k > 20: break

    summary_df = pd.DataFrame(summary_data)
    
    # Печать финальной таблицы в терминал
    print("\nФИНАЛЬНАЯ ТАБЛИЦА РЕЗУЛЬТАТОВ:")
    print(summary_df.to_string(index=False))
    print(f"X_k = {x_k}")

    final_results = {'x_star': np.round(x_k, 4), 'f_val': f(x_k), 'g_val': g(x_k)}
    params = {'x_init': x_init, 'mu_init': mu_init, 'beta': beta, 'epsilon': epsilon}

    plot_optimization_process(trajectory, final_results['x_star'])
   
    # Генерация отчета
    generate_docx(all_iterations_for_report, summary_df, final_results, params)

if __name__ == "__main__":
    solve_barrier_method(x_init=[-1, 2], mu_init=10, beta=0.1, epsilon=0.5, strategy=a)
