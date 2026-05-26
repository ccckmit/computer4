module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;

    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule

module Adder4(a, b, cin, sum, cout);
    input [3:0] a, b;
    input cin;
    output [3:0] sum;
    output cout;
    wire [3:0] c;

    FullAdder fa0(.a(a[0]), .b(b[0]), .cin(cin), .sum(sum[0]), .cout(c[0]));
    FullAdder fa1(.a(a[1]), .b(b[1]), .cin(c[0]), .sum(sum[1]), .cout(c[1]));
    FullAdder fa2(.a(a[2]), .b(b[2]), .cin(c[1]), .sum(sum[2]), .cout(c[2]));
    FullAdder fa3(.a(a[3]), .b(b[3]), .cin(c[2]), .sum(sum[3]), .cout(cout));
endmodule

module Adder4_tb;
    reg [3:0] a, b;
    reg cin;
    wire [3:0] sum;
    wire cout;

    Adder4 dut(a, b, cin, sum, cout);

    initial begin
        $display("=== Adder4 Testbench ===");
        a = 4'd3; b = 4'd5; cin = 1'b0;
        #10;
        $display("3 + 5 + 0 = %d (carry=%d)", sum, cout);
        a = 4'd9; b = 4'd7; cin = 1'b1;
        #10;
        $display("9 + 7 + 1 = %d (carry=%d)", sum, cout);
        a = 4'd0; b = 4'd15; cin = 1'b0;
        #10;
        $display("0 + 15 + 0 = %d (carry=%d)", sum, cout);
        a = 4'd15; b = 4'd15; cin = 1'b1;
        #10;
        $display("15 + 15 + 1 = %d (carry=%d)", sum, cout);
        $finish;
    end
endmodule
