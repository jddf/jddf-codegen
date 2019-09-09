export interface Ref {
  a: string;
}

export interface GamutValues {
  a: string;
}

export interface GamutDiscriminatorB {
  tag: "b";
  b: string;
}

export interface GamutDiscriminatorA {
  tag: "a";
  a: string;
}

export interface GamutType {
  b: string;
  d: number;
  f: number;
  c: string;
  e: number;
  j: number;
  a: boolean;
  h: number;
  k: number;
  g: number;
  i: number;
}

export interface GamutElements {
  a: string;
}

export interface Gamut {
  enum: "BAZ" | "BAR" | "FOO";
  values: { [name: string]: GamutValues};
  discriminator: GamutDiscriminatorB | GamutDiscriminatorA;
  empty: any;
  type: GamutType;
  elements: GamutElements[];
  ref: Ref;
}

