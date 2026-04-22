import pygame
from pygame.math import Vector2 as vec2
from pygame.math import Vector3 as vec3
from pygame import *
from random import randrange
import math
pygame.init()
sw,sh = 800,600
screen = pygame.display.set_mode([sw,sh])
pygame.display.set_caption("Fluid simulation")
clock = pygame.time.Clock()
running = True

class Particle:
    def __init__(self, pos: vec2):
        self.pos = pos
        self.vel = vec2(0,0)
        self.d = 0
        
particles = [Particle(vec2(randrange(0, sw), randrange(0, sh))) for _ in range(100)]
smooth_rad = 40
target_density = .1
pressureMult = 100000 * (smooth_rad / 100)**2

def density_to_pressure(d):
    return (target_density-d) * pressureMult

def poly6_kernel(r, h):
    if r >= h:
        return 0
    h2 = h * h
    coef = 315 / (64 * math.pi * h2**3)  # h^6 instead of h^9
    return coef * (h2 - r * r)**3

def smooth_rad_slope(r, h):    
    if r >= h:
        return 0
    coef = -45 / (math.pi * h**5)
    return coef * (h - r)**2

def calc_density(pos: vec2):
    d = 0
    for j,p2 in enumerate(particles):
        d += poly6_kernel(pos.distance_to(p2.pos), smooth_rad)
    # print(d)
    return d

def calc_pressure_force(p_idx) -> vec2:
    p1 = particles[p_idx]
    pressure_force = vec2()
    for i,p2 in enumerate(particles):
        if i == p_idx:continue
        dst = p2.pos.distance_to(p1.pos)
        if dst == 0: continue
        dir = (p2.pos-p1.pos)/dst
        slope = smooth_rad_slope(dst, smooth_rad)
        shared_pressure = (density_to_pressure(p1.d)+density_to_pressure(p2.d))/2
        # print(shared_pressure, slope, p2.d)
        pressure_force += shared_pressure * dir * slope / max(p2.d, 0.1)
        
    return pressure_force
def wall_repulsion_force(p, strength=5, buffer=40):
    force = vec2(0, 0)
    
    # Left Wall
    if p.pos.x < buffer:
        dist = p.pos.x / buffer  # Normalize distance (0 to 1)
        force.x += strength * (1 - dist)**2  # Quadratic push
    
    # Right Wall
    if p.pos.x > sw - buffer:
        dist = (sw - p.pos.x) / buffer
        force.x -= strength * (1 - dist)**2  

    # Top Wall
    if p.pos.y < buffer:
        dist = p.pos.y / buffer
        force.y += strength * (1 - dist)**2  

    # Bottom Wall
    if p.pos.y > sh - buffer:
        dist = (sh - p.pos.y) / buffer
        force.y -= strength * (1 - dist)**2  

    return -force

while running:
    dt = clock.tick(60)
    screen.fill((255,255,255))
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False
        if event.type == pygame.MOUSEBUTTONUP:
            mx,my = pygame.mouse.get_pos()
    
    new_particles = particles.copy()

    for i,p1 in enumerate(particles):
        p1.d = calc_density(p1.pos)
        p1.vel.y += 0.1
        # print(p1.d)
    # for x in range(400):
    #     for y in range(60):
    #         d = calc_density(vec2(x*2,y*10))
    #         pygame.draw.rect(screen, Color(int(max(0, min(255, d))), 0, int(max(0, min(255, -d)))), (x*2, y*10, 2, 10))

    particles = new_particles
    for i,p in enumerate(particles):
        force = calc_pressure_force(i)
        acc = (force + wall_repulsion_force(p))/p.d # p.d approximated to mass because fluid
        # print(force, p.d, acc)
        # print(acc)
        # print(force, acc)
        p.vel += -acc # !!!!!!!! +=
        p.vel *= 0.99
        p.pos += p.vel
        # if p.pos.x>sw:p.pos.x=sw;p.vel.x*=-0.5
        # if p.pos.y>sh:p.pos.y=sh;p.vel.y*=-0.5
        # if p.pos.x<0:p.pos.x=0;p.vel.x*=-0.5
        # if p.pos.y<0:p.pos.y=0;p.vel.y*=-0.5
        # print(p.d)
        pygame.draw.circle(screen, pygame.Color(0, 0, 0), p.pos, smooth_rad/10) # int(max(0, min(255, p.vel.magnitude()*255))),0,int(max(0, min(255, p.d)))
        pygame.draw.circle(screen, pygame.Color(100, 100, 100), p.pos, smooth_rad/10/2)
        pygame.draw.circle(screen, pygame.Color(150,150,150), p.pos, smooth_rad/10/4)
    print(f"{1000/dt}", end="\r")
    pygame.display.update()
pygame.quit()