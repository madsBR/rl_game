import cv2
import os

img = cv2.imread('./imgs/tokens/happy.jpg')
cv2.imwrite('./imgs/tokens/happy.png',img)
img = cv2.imread('./imgs/tokens/goal.jpeg')
cv2.imwrite('./imgs/tokens/goal.png',img)
img = cv2.imread('./imgs/tokens/duncare.jpg')
cv2.imwrite('./imgs/tokens/duncare.png',img)
