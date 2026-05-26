module FactCPU(clk, rst, result, done);
    input clk, rst;
    output reg [15:0] result;
    output reg done;

    reg [15:0] r;
    reg [15:0] sum;
    reg [3:0] i;
    reg [3:0] j;

    always @(*) begin
        if (rst) begin
            result = 0;
            done = 0;
        end else begin
            r = 1;
            for (i = 2; i <= 5; i = i + 1;) begin
                sum = 0;
                for (j = 0; j < i; j = j + 1;) begin
                    sum = sum + r;
                end
                r = sum;
            end
            result = r;
            done = 1;
        end
    end
endmodule
