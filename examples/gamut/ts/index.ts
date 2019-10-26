export interface Ref {
  a: string;
}

export interface GamutType {
  a: boolean;
  c: string;
  j: number;
  k: number;
  b: string;
  f: number;
  d: number;
  h: number;
  e: number;
  g: number;
  i: number;
}

export interface GamutDiscriminatorB {
  tag: "b";
  b: string;
}

export interface GamutDiscriminatorA {
  tag: "a";
  a: string;
}

export interface GamutElements {
  a: string;
}

export interface GamutValues {
  a: string;
}

export interface Gamut {
  type: GamutType;
  ref: Ref;
  enum: "FOO" | "BAR" | "BAZ";
  discriminator: GamutDiscriminatorB | GamutDiscriminatorA;
  elements: GamutElements[];
  empty: any;
  values: { [name: string]: GamutValues};
}

