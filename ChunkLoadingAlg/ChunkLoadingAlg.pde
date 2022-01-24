import java.util.*;

int SCALE = 20;

color state[] = new color[] {
  color(0),
  color(128,0,0),
  color(128,128,0),
  color(0,255,0),
  color(0,128,255),
  color(128,255,255),
  color(255,255,255),
};
int SPREAD = 6;
int MAX = 6; //state.length;

void setup() {
  size(500,500);
  noStroke();
  background(state[0]);
  frameRate(60);
}

Alg alg = new Alg();
void draw() {
  alg.step();
}

void keyPressed() {
  switch (key) {
    case 's':
      frameRate(1); break;
    case 'f':
      frameRate(500); break;
    case 'm':
      frameRate(60); break;
    
  }
}

void mousePressed() {
  if (mouseButton == LEFT)
    alg.setSrc(mouseX/SCALE,mouseY/SCALE);
  else
    alg.removeSrc(mouseX/SCALE,mouseY/SCALE);
}

class Alg {
  
  Deque<PVector> remove = new ArrayDeque();
  Set<PVector> newSrcs = new HashSet();
  Set<PVector> srcs = new HashSet();
  Map<PVector, Integer> states = new HashMap();
  MultiDeque<Tuple<PVector,Integer>> deque = new MultiDeque();
  boolean done = false;
  int lastRank = 0;
  
  void setSrc(int x, int y) {
    PVector p = new PVector(x,y);
    srcs.add(p);
    println("PUSH");
    for (int i = 0; i < MAX; i++) {
      Tuple<PVector,Integer> t = new Tuple(p, SPREAD-i);
      deque.push(i, t);
    }
    done = false;
  }
   
  void step() {
    if (!remove.isEmpty()) {
      stepRemove(); return;
    }
    while (true) {
      
      if (done) return;
      
      int rank = deque.getRank();
      Tuple<PVector,Integer> p = deque.poll();
      if (p == null) {
        
        //if (lastRank == MAX || (srcs.size() == 0 && newSrcs.size() == 0)) {
          println("Done");
          done = true;
          return;
        //}
        
        /*if (newSrcs.size() > 0) {
          srcs.addAll(newSrcs);
          for (PVector s : newSrcs)
            deque.push(0, new Tuple(s, MAX));
          newSrcs.clear();
        }*/
        
        /*for (PVector s : srcs) {
          deque.push(lastRank+1, new Tuple(s, MAX-lastRank));
        }*/
        
        //continue;
      }
      lastRank = rank;
      
      PVector pp = p.a;
      int spread = p.b;
      
      int curState = getState(pp);
      boolean stateTooHigh = curState >= rank;
      if (!stateTooHigh) {
        println(rank);
        setState(pp, rank);
      }
      if (spread > 1)
        for (PVector n : around(pp)) {
          //if (getState(n) >= spread) continue;
          deque.push(rank, new Tuple(n, spread-1));
        }
      if (!stateTooHigh) return;
    }
  }
  
  void removeSrc(int x, int y) {
    PVector p = new PVector(x, y);
    if (srcs.remove(p))
      remove.addLast(p);
  }
  
  void stepRemove() {
    while (true) {
      PVector r = remove.pollFirst();
      if (r == null) break;
      int state = getState(r);
      if (state == 0) continue;
      int max = maxAround(r);
      if (state < max) {
        continue;
      }
      
      setState(r, 0);
      if (state > 1)
        for (PVector n : around(r)) {
          assert (n != null);
          int ns = getState(n);
          if (ns < state)
            remove.addLast(n);
        }
      break;
    }
    if (remove.isEmpty()) {
      for (PVector src : srcs)
        setSrc((int)src.x,(int)src.y);
    }
  }
  
  int maxAround(PVector p) {
    int res = 0;
    for (PVector n : around(p))
      res = max(res, getState(n));
    return res;
  }
  
  PVector[] around(PVector p) {
    PVector a[] = new PVector[8];
    for (int dx = -1; dx <= 1; dx++)
    for (int dy = -1; dy <= 1; dy++) {
      int idx = dx+1+(dy+1)*3;
      if (idx == 4) continue;
      if (idx > 4) idx--;
      a[idx] = new PVector(p.x+dx,p.y+dy);
    }
    return a;
  }
  
  int getState(PVector p) {
    if (p == null) new Exception().printStackTrace();
    return getState(p.x,p.y);
  }
  int getState(float x, float y) {
    return getState((int)x,(int)y);
  }
  int getState(int x, int y) {
    Integer i = states.get(new PVector(x,y));
    if (i == null) return 0;
    return i;
  }
  void setState(PVector p, int i) {
    setState(p.x,p.y,i);
  }
  void setState(float x, float y, int i) {
    setState((int)x,(int)y,i);
  }
  void setState(int x, int y, int i) {
    color col = state[i];
    states.put(new PVector(x,y), i);
    fill(col);
    rect(x*SCALE,y*SCALE,SCALE,SCALE);
  }
  void mark(int x, int y) {
    noFill();
    stroke(255);
    strokeWeight(1);
    rect(x*SCALE+3,y*SCALE+3,SCALE-6,SCALE-6);
    noStroke();
  }
  
}


class MultiDeque<E> {
  
  SortedMap<Integer, Deque<E>> map = new TreeMap();
  
  void pushAll(int rank, Collection<E> es) {
    for (E e : es) push(rank, e);
  }
  
  void push(int rank, E e) {
    Deque<E> deque = map.get(rank);
    if (deque == null) {
      deque = new ArrayDeque();
      map.put(rank, deque);
    }
    deque.addLast(e);
  }
  
  int getRank() {
    if (map.size() == 0) return 0;
    return map.firstKey();
  }
  
  E poll() {
    if (map.size() == 0) return null;
    Integer k = map.firstKey();
    Deque<E> deque = map.get(k);
    E e = deque.pollFirst();
    if (deque.size() == 0)
      map.remove(k);
    return e;
  }
  
  boolean empty() {
    return map.size() == 0;
  }
  
  
}

class Tuple<A,B> {
  A a; B b;
  Tuple(A a, B b) {this.a = a; this.b = b;}
}
