# Reads in a CSV file obtained from http://cvrl.ucl.ac.uk/cmfs.htm
import sys, csv
import numpy as np
from scipy.integrate import trapz, simps

x_table = "const CIE_X: [f32; CIE_SAMPLES] = [\n"
y_table = "const CIE_Y: [f32; CIE_SAMPLES] = [\n"
z_table = "const CIE_Z: [f32; CIE_SAMPLES] = [\n"
y_values = []

reader = csv.reader(sys.stdin)
for row in reader:
    x_table += '\t' + row[1] + ',\n'
    y_table += '\t' + row[2] + ',\n'
    z_table += '\t' + row[3] + ',\n'
    y_values.append(float(row[2]))

x_table += '];'
y_table += '];'
z_table += '];'

# print(x_table)
# print('\n')
# print(y_table)
# print('\n')
# print(z_table)
# print('\n')

# Trapezoid Rule (sanity check)
y_sum = (y_values[0] + y_values[830 - 360]) / 2.0
for i in range(1, 830 - 360):
    y_sum += y_values[i]
print(y_sum)

x = np.array([x for x in range(830 - 360 + 1)]) # fuck i don't know how to use python
y = np.array(y_values)
print(simps(y, x)) # Simpson's rule
print(trapz(y, x)) # Trapezoid rule
