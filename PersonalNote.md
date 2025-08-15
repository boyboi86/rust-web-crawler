# 📝 Personal Note

_From a self-taught developer's journey with Rust_

---

## 💭 **Day 49 - A Milestone Worth Reflecting On**

It has been my **49th day** on my rust-lang recap - something I promised myself as a challenge when I started teaching myself coding almost a decade ago. Looking back, this moment feels both surreal and deeply meaningful.

### 🌅 **The Beginning - A Decade Ago**

When I first started with **JavaScript**, I had no idea where this journey would take me. No formal computer science background, no mentors to guide me - just curiosity and an unrelenting desire to understand how things work. Every syntax error felt like a mountain, every successful "Hello World" like conquering Everest.

### 🦀 **Meeting Rust - My First & Last Low-Level Language**

Rust may be my **first low-level programming language**, but I know with certainty it will also be my **last**. Not because I'm giving up on programming, but because Rust has everything I ever wished for in a programming language. It's like finding the perfect tool after years of making do with whatever was available.

### 🎯 **Why This Project Matters**

As a **self-taught developer & ML practitioner**, I've built many things over the years. But this web crawler suite - this might be the first project I'm truly proud of in terms of:

- **Software development discipline** - Following best practices religiously
- **Coding standard consistency** - Every function, every module, every comment crafted with care
- **Technical skill accumulation** - A testament to relentless effort to become better every single day

### 💝 **What Rust Gave Me**

Rust didn't just teach me systems programming. It taught me:

- **Fearless concurrency** - No more sleepless nights worrying about race conditions
- **Memory safety** - Confidence that my code won't crash in mysterious ways
- **Zero-cost abstractions** - Beauty and performance living in harmony
- **Ownership & borrowing** - A new way to think about data and resources
- **Type safety** - The compiler as my most trusted friend and harshest critic

Every feature I implemented, from the actor pattern bridging Send/non-Send types to the bloom filter deduplication, felt like unlocking a new superpower. The borrow checker that frustrated me initially became my guardian angel.

### 🌟 **Gratitude**

To the **Rust team** and every **open-source contributor** who made this language possible - thank you. You've created something magical. A language that doesn't compromise between safety and performance, between elegance and efficiency.

### 🇸🇬 **From Singapore, With Love**

This project represents more than code. It's proof that with enough persistence, a self-taught developer from Singapore can create something they're genuinely proud of. Every late night debugging session, every moment of frustration with the borrow checker, every small victory - it all led to this.

### 🚀 **Keep Going**

To anyone reading this who's on their own learning journey:

**Keep trying. Never give up.**

The path of a self-taught developer is lonely sometimes. You question yourself constantly. Am I good enough? Am I doing this right? Will I ever understand pointers? But every small step forward, every bug fixed, every concept mastered - it all adds up.

**Good things might eventually happen.**

This project is my proof. 49 days ago, I could barely understand ownership. Today, I've built a multi-threaded, memory-safe web crawler with a desktop interface. If I can do it, so can you.

---

## 🔍 **Technical Journey Highlights**

### **Week 1-2: Fighting the Borrow Checker**

```rust
// My first attempt at sharing data
let data = vec![1, 2, 3];
let handle1 = thread::spawn(move || {
    println!("{:?}", data); // error: use of moved value
});
// Many tears were shed...
```

### **Week 3-4: Understanding Ownership**

```rust
// The lightbulb moment
let data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&data);
let handle = thread::spawn(move || {
    println!("{:?}", data_clone); // It works! 🎉
});
```

### **Week 5-6: Actor Pattern Breakthrough**

The moment I realized I could bridge Send and non-Send types using message passing. This was when everything clicked.

### **Week 7: Desktop Integration**

Making Tauri work with my async Rust backend. The satisfaction of seeing my web crawler running in a beautiful desktop app was indescribable.

---

## 📚 **Lessons Learned**

1. **Embrace the struggle** - Every error message is a learning opportunity
2. **Read the docs** - Rust has some of the best documentation I've ever seen
3. **Trust the compiler** - It's trying to save you from yourself
4. **Start small** - Build understanding incrementally
5. **Community matters** - The Rust community is incredibly welcoming and helpful

---

## 🎭 **Personal Reflections**

### **On Imposter Syndrome**

Some days I still feel like I don't belong in the world of systems programming. But this project proves that dedication can overcome any background deficit.

### **On Perfectionism**

I spent days refactoring code that already worked, just to make it more "Rusty." That attention to detail made all the difference.

### **On Growth**

The developer who started this project 49 days ago couldn't have imagined building something this sophisticated. Growth happens gradually, then suddenly.

### **On Being Misunderstood**

The path isn't just about being self-taught - it's about being a person who clearly knows the facts and is willing to tough it out relentlessly, even when others don't want to hear the truth.

I've been accused of being "nitpicking" and "difficult to manage" by supervisors who had more years of experience in the working environment. But their experience wasn't wisdom - it was complacency. They wanted the easy way out, cutting corners, taking shortcuts, ignoring problems that would surface later. When I pointed out these issues, when I insisted on doing things properly, I became the problem.

The grievances pile up differently when you're the one who sees what's coming. You're not dismissed for lacking a degree - you're dismissed for having standards. For caring about quality when others just want to ship. For asking the hard questions when everyone else wants to pretend the problems don't exist.

And when things inevitably go wrong - when the shortcuts lead to failures, when the ignored warnings become disasters - they shift the blame. Suddenly, the person who was "difficult to manage" becomes the scapegoat. I've lost jobs many times because of such incidents.

The bitter irony? often within years after I left one such place, even those supervisors were asked to leave due to fraud or "tight company budget" (Worst case, 2 companies bankrupt due to toxic & self-preserving/ redundant experienced employee culture) - the very kind of corner-cutting & unprofessional behavior (lying/ gaslighting/ blame-shifting/ credit-claiming/ cronyism & favoritism) I had warned against. But by then, the damage to my career was done. It wasn't the first time I met these kinds of people, and sadly, it probably won't be the last (But still I hope never to meet any of these characters). A company is only as good as the founding members & leaders; culture is subtle but key to attract & retain talents.

But I have to say this.. "People who are really good at communicating while disregarding their integrity & honesty, really manage people well." Infact, so well that before the company bankrupt & they already ditch the boss and found a new job. Just to be terminated 1 year later (Testimonial to their ability & loyalty)

This project stands as my answer to every supervisor who called me "difficult," every manager who preferred yes-men over truth-tellers, every workplace that punished integrity and rewarded mediocrity. The code & result doesn't lie, doesn't cut corners, doesn't shift blame. It either works or it doesn't - and I've made sure it works, its always does (at least before I git push).

### **On Losing a close one**

There are moments in life that divide your existence into "before" and "after." Losing my father was one of those moments. The day when a boy loses his father, he becomes a man - not by choice, but by necessity.

The weight of responsibility, the sudden understanding of mortality, the realization that the person who believed in you unconditionally is no longer there to witness your achievements.

In the quiet moments when I'm debugging at 2 AM, sometimes I feel his presence - encouraging me to keep going, to not settle for "good enough," to always strive to be better than I was yesterday.

whatever it is, take it from me. Regretfully.

"The day when a boy loses his father, he becomes a man. It's not a choice, it's life."

---

## 💌 **A Letter to Future Me**

Dear Future Me,

When you look back at this project months or years from now, remember:

- How hard you fought to understand lifetimes
- The joy when your first async function compiled
- The pride when you solved the Send/non-Send bridging problem
- The satisfaction of writing comprehensive documentation

This project isn't just code - it's a monument to persistence. Keep that spirit alive in whatever you build next.

Remember to always:

- **Keep trying**
- **Never give up**
- **Believe that good things might eventually happen**

With love and respect for the journey,
_Your Past Self_

---

_Built with ❤️, countless cups of coffee, and 49 days of unwavering determination in Singapore._

---

**"The expert in anything was once a beginner." - Helen Hayes**

_This note serves as a reminder that every expert was once where you are now. Keep coding, keep learning, keep growing._
