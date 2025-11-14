/*---------------------------------------*\
| Simple direct drive of Bipolar Stepper  |
\*---------------------------------------*/

// pins connected
#define Ap 10  // A+ line
#define Am 11  // A- line
#define Bp 9   //  .
#define Bm 8   //

void setup() {
  pinMode(Ap,OUTPUT);  pinMode(Am,OUTPUT);
  pinMode(Bp,OUTPUT);  pinMode(Bm,OUTPUT);
}

int ms = 10;
// int I = 0 ;
void loop() {

  // This is Wave Fullstep motion - only one coil energized at a time
  // if ( I++ <25 ) {  // stop after 25*4 steps (=100) - quarter turn with gearRatio "2:1"
    digitalWrite(Bm,LOW) ;  digitalWrite(Ap,HIGH);
    delay(ms);
    digitalWrite(Ap,LOW) ;  digitalWrite(Bp,HIGH);
    delay(ms);
    digitalWrite(Bp,LOW) ;  digitalWrite(Am,HIGH);
    delay(ms);
    digitalWrite(Am,LOW) ;  digitalWrite(Bm,HIGH);
    delay(ms);
    // } else while(1);
}
