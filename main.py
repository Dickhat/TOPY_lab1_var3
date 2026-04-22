import numpy as np
import pandas as pd
from docx import Document
from docx.shared import Pt
from docx.enum.text import WD_ALIGN_PARAGRAPH

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
# ФОРМИРОВАНИЕ ОТЧЕТА DOCX
# =====================================================================

def generate_docx(all_data, final_results, params):
    doc = Document()
    
    # Заголовок
    title = doc.add_heading('Отчет по методу барьерных функций', 0)
    title.alignment = WD_ALIGN_PARAGRAPH.CENTER

    # Параметры
    doc.add_heading('1. Исходные параметры', level=1)
    p = doc.add_paragraph()
    p.add_run(f"Начальная точка X0: {params['x_init']}\n")
    p.add_run(f"Начальный параметр mu: {params['mu_init']}\n")
    p.add_run(f"Коэффициент уменьшения beta: {params['beta']}\n")
    p.add_run(f"Точность epsilon: {params['epsilon']}")

    # Итерации
    doc.add_heading('2. Ход решения', level=1)
    
    for iteration_data in all_data:
        doc.add_heading(f"Внешняя итерация {iteration_data['k']} (mu = {iteration_data['mu']})", level=2)
        
        # Создание таблицы
        df = iteration_data['table']
        table = doc.add_table(rows=1, cols=len(df.columns))
        table.style = 'Table Grid'
        
        # Заголовки таблицы
        hdr_cells = table.rows[0].cells
        for i, column_name in enumerate(df.columns):
            hdr_cells[i].text = column_name

        # Данные таблицы
        for _, row in df.iterrows():
            row_cells = table.add_row().cells
            for i, value in enumerate(row):
                row_cells[i].text = str(value)
        
        doc.add_paragraph(f"Проверка критерия: mu*B(x) = {iteration_data['check']:.4f}")

    # Итог
    doc.add_heading('3. Итоговый результат', level=1)
    res_p = doc.add_paragraph()
    res_p.add_run(f"Оптимальная точка X*: {final_results['x_star']}\n")
    res_p.add_run(f"Значение целевой функции f(X*): {final_results['f_val']:.6f}\n")
    res_p.add_run(f"Значение ограничения g(X*): {final_results['g_val']:.6f}")

    filename = "Optimization_Report.docx"
    doc.save(filename)
    print(f"\nОтчет успешно сохранен в файл: {filename}")

# =====================================================================
# ОСНОВНОЙ ЦИКЛ
# =====================================================================

def solve_barrier_method(x_init, mu_init, beta, epsilon):
    x_k = np.array(x_init, dtype=float)
    mu = mu_init
    k = 1
    
    all_iterations_for_report = []
    params = {'x_init': x_init, 'mu_init': mu_init, 'beta': beta, 'epsilon': epsilon}

    print(f"--- ЗАПУСК АЛГОРИТМА ---")
    print(f"Параметры: mu={mu_init}, beta={beta}, eps={epsilon}, X0={x_init}\n")

    while True:
        print(f"\n>>> ВНЕШНЯЯ ИТЕРАЦИЯ {k} (mu = {mu:.4f})")
        
        # Шаг 1: Находим минимум P(x, mu)
        x_next, table = coordinate_descent(x_k, mu, 0.0001)
        
        # Шаг 2: Проверка критерия остановки
        val_kBx = -mu / g(x_next)
        print(f"\nПроверка критерия: mu*B(x) = {val_kBx:.4f} (нужно < {epsilon})")
        
        # Сохраняем данные для отчета
        all_iterations_for_report.append({
            'k': k,
            'mu': mu,
            'table': table,
            'check': val_kBx
        })

        if val_kBx < epsilon:
            print(f"Критерий выполнен на итерации {k}!")
            x_k = x_next
            break
        else:
            print(f"Критерий не выполнен, продолжаем...")
        
        # Шаг 3: Обновление параметров
        x_k = x_next
        mu *= beta
        k += 1
        if k > 20: 
            print("Превышено макс. количество внешних итераций.")
            break

    final_results = {
        'x_star': list(np.round(x_k, 4)),
        'f_val': f(x_k),
        'g_val': g(x_k)
    }

    # Вывод финального итога в терминал
    print("\n" + "="*50)
    print("ИТОГОВЫЙ РЕЗУЛЬТАТ:")
    print(f"X* = {final_results['x_star']}")
    print(f"f(X*) = {final_results['f_val']:.6f}")
    print(f"g(X*) = {final_results['g_val']:.6f}")
    print("="*50)

    # Генерация документа
    generate_docx(all_iterations_for_report, final_results, params)

if __name__ == "__main__":
    solve_barrier_method(x_init=[0.0, 12.0], mu_init=10.0, beta=0.1, epsilon=0.5)