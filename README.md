# yew-bots
Живет здесь https://zzzabl.github.io/yew-bots/

Бот управляется скриптом с такими командами
  - if .. endIf - то что внутри отрабатывает если перед ботом нет припятствия
  - step - шаг вперед на одну клетку
  - left - развернуться налево на 90 градусов оставаясь на месте
  - right - развернуться направо на 90 градусов оставаясь на месте
  - leftOrRight - развернуться в случайном направлении на 90 градусов
  - loop .. endLoop - цикл с предусловием. Условие - свободная клетка перед ботом 
  ____
Пример работающего скрипта
```
loop
loop
step
endLoop
leftOrRight
endLoop
left
```
